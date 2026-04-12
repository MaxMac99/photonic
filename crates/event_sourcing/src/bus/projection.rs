use std::fmt::Debug;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

use crate::bus::inmem::{InMemEventBus, SharedEvent};
use crate::bus::subscription::SubscriptionOptions;
use crate::bus::EventBus;
use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::event::event_type::EventType;
use crate::persistence::checkpoint_store::TxCheckpointStore;
use crate::persistence::event_store::EventStore;
use crate::persistence::sequence::Sequence;
use crate::persistence::transaction::TransactionProvider;
use crate::projection::handler::{MultiEventProjectionHandler, ProjectionHandler};
use async_trait::async_trait;
use futures_util::future::join_all;
use snafu::Whatever;
use tokio::task::JoinHandle;
use tokio_stream::Stream;
use tracing::{error, info};

const REPLAY_BATCH_SIZE: usize = 100;

/// Type-erased projection for the engine.
#[async_trait]
trait ErasedProjection<Seq, Tx>: Send + Sync {
    fn name(&self) -> &str;
    fn matches_dyn(&self, event: &dyn DomainEvent) -> bool;
    async fn handle(&self, event: &dyn DomainEvent, sequence: Seq, tx: &mut Tx) -> error::Result<()>;
}

struct SingleHandler<E, Seq, H, Tx>
where
    E: DomainEvent,
    Seq: Sequence,
    H: ProjectionHandler<E, Seq, Tx>,
{
    handler: H,
    event_type: EventType,
    _phantom: std::marker::PhantomData<(E, Seq, Tx)>,
}

#[async_trait]
impl<E, Seq, H, Tx> ErasedProjection<Seq, Tx> for SingleHandler<E, Seq, H, Tx>
where
    E: DomainEvent,
    Seq: Sequence,
    H: ProjectionHandler<E, Seq, Tx>,
    Tx: Send + Sync + 'static,
{
    fn name(&self) -> &str {
        self.handler.name()
    }

    fn matches_dyn(&self, event: &dyn DomainEvent) -> bool {
        self.event_type.id() == (*event).type_id()
    }

    async fn handle(&self, event: &dyn DomainEvent, sequence: Seq, tx: &mut Tx) -> error::Result<()> {
        let any = event as &dyn std::any::Any;
        let typed = any
            .downcast_ref::<E>()
            .expect("type mismatch in projection dispatch");
        self.handler.handle(typed, sequence, tx).await
    }
}

struct MultiHandler<Seq, H, Tx>
where
    Seq: Sequence,
    H: MultiEventProjectionHandler<Seq, Tx>,
{
    handler: H,
    event_types: Vec<EventType>,
    _phantom: std::marker::PhantomData<(Seq, Tx)>,
}

#[async_trait]
impl<Seq, H, Tx> ErasedProjection<Seq, Tx> for MultiHandler<Seq, H, Tx>
where
    Seq: Sequence,
    H: MultiEventProjectionHandler<Seq, Tx>,
    Tx: Send + Sync + 'static,
{
    fn name(&self) -> &str {
        self.handler.name()
    }

    fn matches_dyn(&self, event: &dyn DomainEvent) -> bool {
        let type_id = (*event).type_id();
        self.event_types.iter().any(|et| et.id() == type_id)
    }

    async fn handle(&self, event: &dyn DomainEvent, sequence: Seq, tx: &mut Tx) -> error::Result<()> {
        self.handler.handle(event, sequence, tx).await
    }
}

/// Event bus that runs projections synchronously in parallel within transactions,
/// then dispatches events to downstream consumers via an in-memory bus.
///
/// Flow for each published event:
/// 1. Append event to the event store.
/// 2. For each matching projection, in parallel: begin transaction, run the
///    projection, save its checkpoint, commit.
/// 3. Dispatch the event to the in-memory bus for non-projection consumers.
///
/// On `start()`, replays events from the earliest projection checkpoint,
/// running each event through all projections (in transactions) until caught
/// up. Projections that fail during replay are retried on the next run.
pub struct ProjectionEventBus<Seq, Tx> {
    store: Arc<dyn EventStore<Seq>>,
    checkpoint_store: Arc<dyn TxCheckpointStore<Seq, Tx>>,
    tx_provider: Arc<dyn TransactionProvider<Tx>>,
    inmem: Arc<InMemEventBus>,
    projections: std::sync::Mutex<Vec<Arc<dyn ErasedProjection<Seq, Tx>>>>,
    started: AtomicBool,
}

impl<Seq, Tx> ProjectionEventBus<Seq, Tx>
where
    Seq: Sequence + Debug,
    Tx: Send + Sync + 'static,
{
    pub fn new(
        store: impl EventStore<Seq>,
        checkpoint_store: impl TxCheckpointStore<Seq, Tx>,
        tx_provider: impl TransactionProvider<Tx>,
    ) -> Self {
        Self {
            store: Arc::new(store),
            checkpoint_store: Arc::new(checkpoint_store),
            tx_provider: Arc::new(tx_provider),
            inmem: Arc::new(InMemEventBus::new()),
            projections: std::sync::Mutex::new(Vec::new()),
            started: AtomicBool::new(false),
        }
    }

    /// Register a single-event projection. Must be called before `start()`.
    pub fn register<E, H>(&self, handler: H) -> error::Result<()>
    where
        E: DomainEvent,
        H: ProjectionHandler<E, Seq, Tx>,
    {
        if self.started.load(AtomicOrdering::SeqCst) {
            return Err(error::EventSourcingError::Bus {
                message: "Cannot register projection after bus has started".into(),
            });
        }
        self.projections.lock().unwrap().push(Arc::new(SingleHandler {
            handler,
            event_type: EventType::of::<E>(),
            _phantom: std::marker::PhantomData,
        }));
        Ok(())
    }

    /// Register a multi-event projection. Must be called before `start()`.
    pub fn register_multi<H>(&self, handler: H) -> error::Result<()>
    where
        H: MultiEventProjectionHandler<Seq, Tx>,
    {
        if self.started.load(AtomicOrdering::SeqCst) {
            return Err(error::EventSourcingError::Bus {
                message: "Cannot register projection after bus has started".into(),
            });
        }
        let event_types = handler.event_types();
        self.projections.lock().unwrap().push(Arc::new(MultiHandler {
            handler,
            event_types,
            _phantom: std::marker::PhantomData,
        }));
        Ok(())
    }

    /// Starts the bus. Runs the replay phase synchronously to ensure all
    /// projections are caught up before accepting publishes.
    /// Returns `Err` if called more than once.
    pub async fn start(&self) -> error::Result<()> {
        if self.started.swap(true, AtomicOrdering::SeqCst) {
            return Err(error::EventSourcingError::Bus {
                message: "ProjectionEventBus::start() called more than once".into(),
            });
        }

        let projections: Vec<Arc<dyn ErasedProjection<Seq, Tx>>> =
            self.projections.lock().unwrap().clone();

        if projections.is_empty() {
            info!("No projections registered, skipping replay");
            return Ok(());
        }

        // Load each projection's checkpoint. Each (projection, checkpoint)
        // pair is tracked independently — a projection only receives events
        // with sequence > its own checkpoint.
        let mut tracked: Vec<(Arc<dyn ErasedProjection<Seq, Tx>>, Seq)> =
            Vec::with_capacity(projections.len());
        for projection in projections {
            let mut tx = self.tx_provider.begin().await?;
            let seq = self.checkpoint_store.load(projection.name(), &mut tx).await?;
            self.tx_provider.commit(tx).await?;
            tracked.push((projection, seq));
        }

        // Find the earliest checkpoint — where replay starts.
        let replay_start = tracked
            .iter()
            .map(|(_, seq)| *seq)
            .reduce(|a, b| if a.is_behind(&b) { a } else { b })
            .unwrap_or_default();

        info!(
            projections = tracked.len(),
            "Starting projection replay phase"
        );

        let mut last_seq = replay_start;
        loop {
            let stored_events = self
                .store
                .load(last_seq, vec![], REPLAY_BATCH_SIZE)
                .await?;

            if stored_events.is_empty() {
                break;
            }

            for stored in stored_events {
                let seq = stored.sequence;
                let shared: SharedEvent = Arc::from(stored.event);

                // Only dispatch to projections whose checkpoint is behind this event
                let active: Vec<Arc<dyn ErasedProjection<Seq, Tx>>> = tracked
                    .iter()
                    .filter(|(_, cp)| cp.is_behind(&seq))
                    .map(|(p, _)| p.clone())
                    .collect();

                self.run_projections(&active, &shared, seq).await;

                // Advance tracked checkpoints for projections that just ran
                for (projection, cp) in tracked.iter_mut() {
                    if cp.is_behind(&seq) && projection.matches_dyn(shared.as_ref()) {
                        *cp = seq;
                    }
                }

                if last_seq.is_behind(&seq) {
                    last_seq = seq;
                }
            }
        }

        info!("Projection replay complete");
        Ok(())
    }

    /// Runs all matching projections for an event in parallel, each within
    /// its own transaction. Checkpoints are saved atomically with projection
    /// work via the transactional checkpoint store.
    async fn run_projections(
        &self,
        projections: &[Arc<dyn ErasedProjection<Seq, Tx>>],
        event: &SharedEvent,
        sequence: Seq,
    ) {
        let futures = projections
            .iter()
            .filter(|p| p.matches_dyn(event.as_ref()))
            .map(|p| {
                let p = p.clone();
                let tx_provider = self.tx_provider.clone();
                let checkpoint_store = self.checkpoint_store.clone();
                let event = event.clone();
                async move {
                    Self::run_projection(p, tx_provider, checkpoint_store, event, sequence).await
                }
            });

        join_all(futures).await;
    }

    async fn run_projection(
        projection: Arc<dyn ErasedProjection<Seq, Tx>>,
        tx_provider: Arc<dyn TransactionProvider<Tx>>,
        checkpoint_store: Arc<dyn TxCheckpointStore<Seq, Tx>>,
        event: SharedEvent,
        sequence: Seq,
    ) {
        let mut tx = match tx_provider.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!(error = %e, projection = projection.name(), "Failed to begin transaction");
                return;
            }
        };

        if let Err(e) = projection.handle(event.as_ref(), sequence, &mut tx).await {
            error!(error = %e, projection = projection.name(), "Projection failed");
            if let Err(e) = tx_provider.rollback(tx).await {
                error!(error = %e, "Failed to rollback transaction");
            }
            return;
        }

        if let Err(e) = checkpoint_store.save(projection.name(), sequence, &mut tx).await {
            error!(error = %e, projection = projection.name(), "Failed to save checkpoint");
            if let Err(e) = tx_provider.rollback(tx).await {
                error!(error = %e, "Failed to rollback transaction");
            }
            return;
        }

        if let Err(e) = tx_provider.commit(tx).await {
            error!(error = %e, projection = projection.name(), "Failed to commit transaction");
        }
    }
}

#[async_trait]
impl<Seq, Tx> EventBus for ProjectionEventBus<Seq, Tx>
where
    Seq: Sequence + Debug,
    Tx: Send + Sync + 'static,
{
    async fn publish(&self, event: impl DomainEvent) -> error::Result<()> {
        // 1. Append to store
        let seq = self.store.append(&event).await?;

        // 2. Run projections in parallel
        let shared: SharedEvent = Arc::new(event);
        let projections: Vec<Arc<dyn ErasedProjection<Seq, Tx>>> =
            self.projections.lock().unwrap().clone();
        self.run_projections(&projections, &shared, seq).await;

        // 3. Dispatch to inmem for non-projection consumers
        if let Err(e) = self.inmem.publish_shared(shared).await {
            error!(error = %e, "Failed to dispatch event to inmem");
        }

        Ok(())
    }

    async fn subscribe<E>(&self) -> impl Stream<Item = Arc<E>>
    where
        E: DomainEvent,
    {
        self.inmem.subscribe::<E>().await
    }

    async fn start_consumer_with_options<E, F, Fut>(
        &self,
        options: SubscriptionOptions,
        consumer: F,
    ) -> Result<Vec<JoinHandle<()>>, Whatever>
    where
        E: DomainEvent,
        F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.inmem
            .start_consumer_with_options(options, consumer)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EventSourcingError;
    use crate::event::domain_event::fixtures::{StoredTestEvent, TestEvent};
    use crate::persistence::checkpoint_store::fixtures::MockTxCheckpointStore;
    use crate::persistence::event_store::fixtures::MockEventStore;
    use crate::persistence::transaction::fixtures::{MockTx, MockTxProvider};
    use futures_util::StreamExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{timeout, Duration};

    // -- Test-specific projection handlers --

    struct CountingProjection<E: DomainEvent> {
        name: String,
        count: Arc<AtomicUsize>,
        _phantom: std::marker::PhantomData<E>,
    }

    impl<E: DomainEvent> CountingProjection<E> {
        fn new(name: &str, count: Arc<AtomicUsize>) -> Self {
            Self {
                name: name.to_string(),
                count,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    #[async_trait]
    impl<E: DomainEvent> ProjectionHandler<E, i64, MockTx> for CountingProjection<E> {
        fn name(&self) -> &str {
            &self.name
        }

        async fn handle(&self, _event: &E, _sequence: i64, _tx: &mut MockTx) -> error::Result<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct FailingProjection {
        name: String,
    }

    #[async_trait]
    impl ProjectionHandler<TestEvent, i64, MockTx> for FailingProjection {
        fn name(&self) -> &str {
            &self.name
        }

        async fn handle(&self, _event: &TestEvent, _sequence: i64, _tx: &mut MockTx) -> error::Result<()> {
            Err(EventSourcingError::Store {
                message: "projection failed".into(),
            })
        }
    }

    fn make_bus() -> ProjectionEventBus<i64, MockTx> {
        ProjectionEventBus::new(
            MockEventStore::new(),
            MockTxCheckpointStore::new(),
            MockTxProvider::new(),
        )
    }

    // -- Tests --

    #[tokio::test]
    async fn publish_runs_projection() {
        let bus = make_bus();
        let count = Arc::new(AtomicUsize::new(0));
        bus.register(CountingProjection::<TestEvent>::new("proj", count.clone()))
            .unwrap();
        bus.start().await.unwrap();

        bus.publish(TestEvent::new("hello")).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn projections_run_in_parallel() {
        let bus = make_bus();
        let count_a = Arc::new(AtomicUsize::new(0));
        let count_b = Arc::new(AtomicUsize::new(0));
        bus.register(CountingProjection::<TestEvent>::new("a", count_a.clone()))
            .unwrap();
        bus.register(CountingProjection::<TestEvent>::new("b", count_b.clone()))
            .unwrap();
        bus.start().await.unwrap();

        bus.publish(TestEvent::new("hello")).await.unwrap();

        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn replay_runs_projections_for_existing_events() {
        let store = MockEventStore::new();
        store.append(&TestEvent::new("first")).await.unwrap();
        store.append(&TestEvent::new("second")).await.unwrap();

        let bus = ProjectionEventBus::new(
            store,
            MockTxCheckpointStore::new(),
            MockTxProvider::new(),
        );

        let count = Arc::new(AtomicUsize::new(0));
        bus.register(CountingProjection::<StoredTestEvent>::new(
            "proj",
            count.clone(),
        ))
        .unwrap();

        bus.start().await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn replay_resumes_from_checkpoint() {
        let store = MockEventStore::new();
        store.append(&TestEvent::new("first")).await.unwrap();
        store.append(&TestEvent::new("second")).await.unwrap();
        store.append(&TestEvent::new("third")).await.unwrap();

        let checkpoint_store = MockTxCheckpointStore::new();
        checkpoint_store
            .checkpoints
            .lock()
            .unwrap()
            .insert("proj".to_string(), 2);

        let bus = ProjectionEventBus::new(
            store,
            checkpoint_store,
            MockTxProvider::new(),
        );

        let count = Arc::new(AtomicUsize::new(0));
        bus.register(CountingProjection::<StoredTestEvent>::new(
            "proj",
            count.clone(),
        ))
        .unwrap();

        bus.start().await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1, "only event after checkpoint");
    }

    #[tokio::test]
    async fn checkpoint_saved_after_successful_projection() {
        let bus = make_bus();
        let count = Arc::new(AtomicUsize::new(0));
        bus.register(CountingProjection::<TestEvent>::new("proj", count.clone()))
            .unwrap();
        bus.start().await.unwrap();

        bus.publish(TestEvent::new("hello")).await.unwrap();

        let saved = bus
            .checkpoint_store
            .load("proj", &mut MockTx)
            .await
            .unwrap();
        assert_eq!(saved, 1);
    }

    #[tokio::test]
    async fn checkpoint_not_saved_on_projection_failure() {
        let bus = make_bus();
        bus.register(FailingProjection {
            name: "failing".into(),
        })
        .unwrap();
        bus.start().await.unwrap();

        bus.publish(TestEvent::new("will fail")).await.unwrap();

        let saved = bus
            .checkpoint_store
            .load("failing", &mut MockTx)
            .await
            .unwrap();
        assert_eq!(saved, 0);
    }

    #[tokio::test]
    async fn non_projection_consumers_receive_events_after_projections() {
        let bus = make_bus();
        let count = Arc::new(AtomicUsize::new(0));
        bus.register(CountingProjection::<TestEvent>::new("proj", count.clone()))
            .unwrap();
        bus.start().await.unwrap();

        let mut stream = bus.subscribe::<TestEvent>().await;
        bus.publish(TestEvent::new("hello")).await.unwrap();

        let event = timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(event.value, "hello");
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn reject_projection_after_start() {
        let bus = make_bus();
        bus.start().await.unwrap();

        let result = bus.register(CountingProjection::<TestEvent>::new(
            "late",
            Arc::new(AtomicUsize::new(0)),
        ));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn start_can_only_be_called_once() {
        let bus = make_bus();
        assert!(bus.start().await.is_ok());
        assert!(bus.start().await.is_err());
    }
}