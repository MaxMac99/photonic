use std::fmt::Debug;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

use crate::bus::inmem::{InMemEventBus, SharedEvent};
use crate::bus::subscription::{StartFrom, SubscriptionOptions};
use crate::bus::EventBus;
use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::event::event_metadata::EventMetadata;
use crate::persistence::checkpoint_store::CheckpointStore;
use crate::persistence::event_store::EventStore;
use crate::persistence::sequence::Sequence;
use async_trait::async_trait;
use snafu::Whatever;
use tokio::task::JoinHandle;
use tokio_stream::Stream;
use tracing::{error, info, warn};

const REPLAY_BATCH_SIZE: usize = 100;

/// Wrapper that carries a sequence alongside an event. Published to inmem
/// so that replayable consumer wrappers can extract the sequence for
/// checkpoint saving. Each `E` produces a distinct `TypeId` for routing.
#[derive(Debug)]
struct SequencedEvent<Seq, E> {
    sequence: Seq,
    event: Arc<E>,
}

impl<Seq: Debug + Send + Sync + 'static, E: DomainEvent> DomainEvent for SequencedEvent<Seq, E> {
    fn metadata(&self) -> &EventMetadata {
        self.event.metadata()
    }
}

/// Type-erased replayable consumer for the replay phase.
#[async_trait]
trait ReplayableConsumer<Seq>: Send + Sync {
    fn activate_after(&self) -> Seq;
    async fn handle_replay(
        &self,
        event: &SharedEvent,
        seq: Seq,
        checkpoint_store: &dyn CheckpointStore<Seq>,
    );
    async fn register_live(
        self: Box<Self>,
        inmem: &InMemEventBus,
        checkpoint_store: Arc<dyn CheckpointStore<Seq>>,
    ) -> Result<Vec<JoinHandle<()>>, Whatever>;
}

struct TypedConsumer<Seq, E, F, Fut>
where
    E: DomainEvent,
    F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = error::Result<()>> + Send + 'static,
{
    activate_after: Seq,
    checkpoint_name: Option<String>,
    options: SubscriptionOptions,
    consumer: F,
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<Seq, E, F, Fut> ReplayableConsumer<Seq> for TypedConsumer<Seq, E, F, Fut>
where
    Seq: Sequence + Debug,
    E: DomainEvent,
    F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = error::Result<()>> + Send + 'static,
{
    fn activate_after(&self) -> Seq {
        self.activate_after
    }

    async fn handle_replay(
        &self,
        event: &SharedEvent,
        seq: Seq,
        checkpoint_store: &dyn CheckpointStore<Seq>,
    ) {
        let any: Arc<dyn std::any::Any + Send + Sync> = event.clone();
        if let Ok(typed) = Arc::downcast::<E>(any) {
            match (self.consumer)(typed).await {
                Ok(()) => {
                    if let Some(name) = &self.checkpoint_name {
                        if let Err(e) = checkpoint_store.save(name, seq).await {
                            error!(error = %e, consumer = name.as_str(), "Failed to save checkpoint");
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Consumer failed during replay");
                }
            }
        }
    }

    async fn register_live(
        self: Box<Self>,
        inmem: &InMemEventBus,
        checkpoint_store: Arc<dyn CheckpointStore<Seq>>,
    ) -> Result<Vec<JoinHandle<()>>, Whatever> {
        let consumer = self.consumer.clone();
        let checkpoint_name = self.checkpoint_name.clone();

        // Wrap: subscribe to SequencedEvent<Seq, E>, extract inner event,
        // call consumer, save checkpoint on success.
        let wrapped = move |sequenced: Arc<SequencedEvent<Seq, E>>| {
            let consumer = consumer.clone();
            let checkpoint_store = checkpoint_store.clone();
            let checkpoint_name = checkpoint_name.clone();
            async move {
                let seq = sequenced.sequence;
                let event = sequenced.event.clone();
                match consumer(event).await {
                    Ok(()) => {
                        if let Some(name) = &checkpoint_name {
                            if let Err(e) = checkpoint_store.save(name, seq).await {
                                error!(error = %e, consumer = name.as_str(), "Failed to save checkpoint");
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Consumer failed to process event");
                    }
                }
            }
        };

        inmem
            .start_consumer_with_options(self.options.clone(), wrapped)
            .await
    }
}

/// Event bus that persists events and replays from checkpoints on startup.
///
/// Lifecycle:
/// 1. **Registration** — `start_consumer_with_options` registers `Latest`
///    consumers directly on inmem. Replayable consumers (`Beginning`/`Checkpoint`)
///    are registered via `start_replayable_consumer` and queued for replay.
/// 2. **Replay** (`start()`) — spawns a background task that replays events
///    directly to consumer callbacks. After replay, consumers are registered
///    on inmem for live events via `SequencedEvent<Seq, E>` wrappers.
/// 3. **Live** — `publish` dispatches both `Arc<E>` (for Latest consumers)
///    and `Arc<SequencedEvent<Seq, E>>` (for replayable consumers) to inmem.
pub struct PersistentEventBus<Seq> {
    store: Arc<dyn EventStore<Seq>>,
    checkpoint_store: Arc<dyn CheckpointStore<Seq>>,
    inmem: Arc<InMemEventBus>,
    pending: std::sync::Mutex<Vec<Box<dyn ReplayableConsumer<Seq>>>>,
    started: AtomicBool,
}

impl<Seq> PersistentEventBus<Seq>
where
    Seq: Sequence + Debug,
{
    pub fn new(
        store: impl EventStore<Seq>,
        checkpoint_store: impl CheckpointStore<Seq>,
    ) -> Self {
        Self {
            store: Arc::new(store),
            checkpoint_store: Arc::new(checkpoint_store),
            inmem: Arc::new(InMemEventBus::new()),
            pending: std::sync::Mutex::new(Vec::new()),
            started: AtomicBool::new(false),
        }
    }

    /// Register a replayable consumer. Must be called before `start()`.
    /// The consumer returns `Result<()>` — checkpoints are saved on success.
    pub async fn start_replayable_consumer<E, F, Fut>(
        &self,
        options: SubscriptionOptions,
        consumer: F,
    ) -> error::Result<()>
    where
        E: DomainEvent,
        F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = error::Result<()>> + Send + 'static,
    {
        if self.started.load(AtomicOrdering::SeqCst) {
            return Err(error::EventSourcingError::Bus {
                message: "Cannot register replayable consumer after bus has started".into(),
            });
        }

        let (activate_after, checkpoint_name) = match &options.start_from {
            StartFrom::Beginning => (Seq::default(), None),
            StartFrom::Checkpoint { consumer_name } => {
                let seq = self
                    .checkpoint_store
                    .load(consumer_name)
                    .await
                    .unwrap_or_else(|e| {
                        warn!(
                            error = %e,
                            consumer = consumer_name.as_str(),
                            "Failed to load checkpoint, starting from beginning"
                        );
                        Seq::default()
                    });
                (seq, Some(consumer_name.clone()))
            }
            StartFrom::Latest => {
                return Err(error::EventSourcingError::Bus {
                    message: "Latest consumers should use start_consumer_with_options".into(),
                });
            }
        };

        self.pending.lock().unwrap().push(Box::new(TypedConsumer {
            activate_after,
            checkpoint_name,
            options,
            consumer,
            _phantom: std::marker::PhantomData::<E>,
        }));

        Ok(())
    }

    /// Starts the bus. Spawns a background replay task if needed.
    /// Returns `Err` if called more than once.
    pub fn start(&self) -> error::Result<JoinHandle<()>> {
        if self.started.swap(true, AtomicOrdering::SeqCst) {
            return Err(error::EventSourcingError::Bus {
                message: "PersistentEventBus::start() called more than once".into(),
            });
        }

        let mut consumers: Vec<Box<dyn ReplayableConsumer<Seq>>> =
            self.pending.lock().unwrap().drain(..).collect();

        consumers.sort_by(|a, b| {
            if a.activate_after().is_behind(&b.activate_after()) {
                std::cmp::Ordering::Less
            } else if b.activate_after().is_behind(&a.activate_after()) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        let store = self.store.clone();
        let checkpoint_store = self.checkpoint_store.clone();
        let inmem = self.inmem.clone();

        Ok(tokio::task::Builder::new()
            .name("persistent-event-bus-replay")
            .spawn(async move {
                Self::replay_task(store, checkpoint_store, inmem, consumers).await;
            })
            .expect("failed to spawn replay task"))
    }

    async fn replay_task(
        store: Arc<dyn EventStore<Seq>>,
        checkpoint_store: Arc<dyn CheckpointStore<Seq>>,
        inmem: Arc<InMemEventBus>,
        mut consumers: Vec<Box<dyn ReplayableConsumer<Seq>>>,
    ) {
        if consumers.is_empty() {
            info!("No replayable consumers, skipping replay");
            return;
        }

        let replay_start = consumers.first().unwrap().activate_after();

        let mut active: Vec<Box<dyn ReplayableConsumer<Seq>>> = Vec::new();
        let mut deferred: Vec<Box<dyn ReplayableConsumer<Seq>>> = Vec::new();

        for consumer in consumers.drain(..) {
            if !replay_start.is_behind(&consumer.activate_after()) {
                active.push(consumer);
            } else {
                deferred.push(consumer);
            }
        }

        let mut last_seq = replay_start;

        info!(
            active_consumers = active.len(),
            deferred_consumers = deferred.len(),
            "Starting replay phase"
        );

        loop {
            let stored_events = match store.load(last_seq, vec![], REPLAY_BATCH_SIZE).await {
                Ok(events) => events,
                Err(e) => {
                    error!(error = %e, "Failed to load events during replay");
                    break;
                }
            };

            if stored_events.is_empty() {
                break;
            }

            for stored in stored_events {
                let seq = stored.sequence;

                let mut i = 0;
                while i < deferred.len() {
                    if deferred[i].activate_after().is_behind(&seq) {
                        active.push(deferred.remove(i));
                    } else {
                        i += 1;
                    }
                }

                let shared: SharedEvent = Arc::from(stored.event);
                for consumer in &active {
                    consumer
                        .handle_replay(&shared, seq, &*checkpoint_store)
                        .await;
                }

                if last_seq.is_behind(&seq) {
                    last_seq = seq;
                }
            }
        }

        // Activate remaining deferred consumers
        active.extend(deferred);

        // Register all on inmem for live events
        for consumer in active {
            if let Err(e) = consumer.register_live(&inmem, checkpoint_store.clone()).await {
                error!(error = %e, "Failed to register consumer for live events");
            }
        }

        info!("Replay complete, transitioned to live mode");
    }
}

#[async_trait]
impl<Seq> EventBus for PersistentEventBus<Seq>
where
    Seq: Sequence + Debug,
{
    async fn publish(&self, event: impl DomainEvent) -> error::Result<()> {
        let seq = self.store.append(&event).await?;
        let event = Arc::new(event);

        // Dispatch SequencedEvent for replayable consumers
        let sequenced: SharedEvent = Arc::new(SequencedEvent {
            sequence: seq,
            event: event.clone(),
        });
        if let Err(e) = self.inmem.publish_shared(sequenced).await {
            error!(error = %e, "Failed to dispatch sequenced event");
        }

        // Dispatch raw event for Latest consumers / subscribe streams
        if let Err(e) = self.inmem.publish_shared(event).await {
            error!(error = %e, "Failed to dispatch event");
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
    use crate::persistence::checkpoint_store::fixtures::MockCheckpointStore;
    use crate::persistence::event_store::fixtures::{FailingEventStore, MockEventStore};
    use futures_util::StreamExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{timeout, Duration};

    fn make_bus(store: MockEventStore) -> PersistentEventBus<i64> {
        PersistentEventBus::new(store, MockCheckpointStore::new())
    }

    // -- Tests --

    #[tokio::test]
    async fn publish_dispatches_to_latest_consumer() {
        let bus = make_bus(MockEventStore::new());
        bus.start().unwrap();

        let mut stream = bus.subscribe::<TestEvent>().await;
        bus.publish(TestEvent::new("hello")).await.unwrap();

        let event = timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(event.value, "hello");
    }

    #[tokio::test]
    async fn publish_returns_error_when_store_fails() {
        let bus = PersistentEventBus::new(FailingEventStore, MockCheckpointStore::new());
        bus.start().unwrap();

        let result = bus.publish(TestEvent::new("will fail")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn beginning_consumer_replays_all_events() {
        let store = MockEventStore::new();
        store.append(&TestEvent::new("first")).await.unwrap();
        store.append(&TestEvent::new("second")).await.unwrap();

        let bus = make_bus(store);

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        bus.start_replayable_consumer::<StoredTestEvent, _, _>(
            SubscriptionOptions::new()
                .named("test")
                .start_from(StartFrom::Beginning),
            move |_| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .await
        .unwrap();

        let handle = bus.start().unwrap();
        handle.await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn checkpoint_consumer_resumes_from_saved_position() {
        let store = MockEventStore::new();
        store.append(&TestEvent::new("first")).await.unwrap();
        store.append(&TestEvent::new("second")).await.unwrap();
        store.append(&TestEvent::new("third")).await.unwrap();

        let cs = MockCheckpointStore::new();
        cs.save("my-consumer", 2).await.unwrap();

        let bus = PersistentEventBus::new(store, cs);

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        bus.start_replayable_consumer::<StoredTestEvent, _, _>(
            SubscriptionOptions::new()
                .named("test")
                .start_from(StartFrom::Checkpoint {
                    consumer_name: "my-consumer".to_string(),
                }),
            move |_| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .await
        .unwrap();

        let handle = bus.start().unwrap();
        handle.await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn checkpoint_saved_on_successful_replay() {
        let store = MockEventStore::new();
        store.append(&TestEvent::new("first")).await.unwrap();
        store.append(&TestEvent::new("second")).await.unwrap();

        let cs = MockCheckpointStore::new();
        let bus = PersistentEventBus::new(store, cs);

        bus.start_replayable_consumer::<StoredTestEvent, _, _>(
            SubscriptionOptions::new()
                .named("test")
                .start_from(StartFrom::Checkpoint {
                    consumer_name: "my-consumer".to_string(),
                }),
            move |_| async { Ok(()) },
        )
        .await
        .unwrap();

        let handle = bus.start().unwrap();
        handle.await.unwrap();

        let saved = bus.checkpoint_store.load("my-consumer").await.unwrap();
        assert_eq!(saved, 2);
    }

    #[tokio::test]
    async fn checkpoint_not_saved_on_consumer_error() {
        let store = MockEventStore::new();
        store.append(&TestEvent::new("first")).await.unwrap();

        let cs = MockCheckpointStore::new();
        let bus = PersistentEventBus::new(store, cs);

        bus.start_replayable_consumer::<StoredTestEvent, _, _>(
            SubscriptionOptions::new()
                .named("test")
                .start_from(StartFrom::Checkpoint {
                    consumer_name: "my-consumer".to_string(),
                }),
            move |_| async {
                Err(EventSourcingError::Store {
                    message: "consumer failed".into(),
                })
            },
        )
        .await
        .unwrap();

        let handle = bus.start().unwrap();
        handle.await.unwrap();

        let saved = bus.checkpoint_store.load("my-consumer").await.unwrap();
        assert_eq!(saved, 0, "checkpoint should not be saved on error");
    }

    #[tokio::test]
    async fn replayable_consumer_receives_live_events_after_replay() {
        let bus = make_bus(MockEventStore::new());

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        bus.start_replayable_consumer::<TestEvent, _, _>(
            SubscriptionOptions::new()
                .named("test")
                .start_from(StartFrom::Beginning),
            move |_| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .await
        .unwrap();

        let handle = bus.start().unwrap();
        handle.await.unwrap();
        tokio::task::yield_now().await;

        bus.publish(TestEvent::new("live")).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn reject_replayable_consumer_after_start() {
        let bus = make_bus(MockEventStore::new());
        bus.start().unwrap();

        let result = bus
            .start_replayable_consumer::<StoredTestEvent, _, _>(
                SubscriptionOptions::new()
                    .named("late")
                    .start_from(StartFrom::Beginning),
                |_| async { Ok(()) },
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn start_can_only_be_called_once() {
        let bus = make_bus(MockEventStore::new());
        assert!(bus.start().is_ok());
        assert!(bus.start().is_err());
    }
}