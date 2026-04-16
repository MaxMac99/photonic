use std::{
    collections::{HashMap, HashSet},
    future::Future,
    pin::pin,
    sync::Arc,
};

use async_trait::async_trait;
use futures_util::StreamExt;
use snafu::Whatever;
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{
    bus::{
        inmem::{InMemEventBus, SharedEvent},
        subscription::SubscriptionOptions,
        EventBus,
    },
    error,
    event::{domain_event::DomainEvent, event_type::EventType},
    persistence::{
        event_store::EventStore, listener::EventListener,
        notifying_event_store::NotifyingEventStore, sequence::Sequence,
    },
};

/// Unique identifier for correlating Pending with Committed/Abort.
type PublishId = u64;

/// Messages sent from the publishing thread to the listener task.
///
/// Two-phase protocol:
/// 1. `Pending` — sent *before* the store append, so the event is in the
///    listener's memory before any DB notification can fire.
/// 2. `Committed` — sent *after* a successful append with the assigned sequence.
/// 3. `Abort` — sent if the append fails, so the listener drops the pending event.
enum PublishMessage<Seq> {
    Pending { id: PublishId, event: SharedEvent },
    Committed { id: PublishId, sequence: Seq },
    Abort { id: PublishId },
}

/// Event bus backed by a [`NotifyingEventStore`].
///
/// Uses a two-phase protocol for local publishes to avoid the
/// notification → load round-trip:
///
/// 1. Before appending, the event is sent to the listener task as `Pending`.
/// 2. After a successful append, `Committed { id, sequence }` is sent.
/// 3. The listener checks whether the committed sequence is contiguous with
///    its `last_sequence`. If so, it dispatches the pending event directly
///    without loading from the store. If there is a gap (remote events arrived
///    in between), it loads the missing events from the store first.
///
/// This guarantees the event is in the listener's memory *before* the DB
/// transaction commits and fires a notification, eliminating the race between
/// the direct channel and the notification path.
pub struct NotifyingPersistentEventBus<Seq, S, L> {
    store: Arc<NotifyingEventStore<S, L>>,
    inmem: Arc<InMemEventBus>,
    publish_tx: async_channel::Sender<PublishMessage<Seq>>,
    publish_rx: async_channel::Receiver<PublishMessage<Seq>>,
    next_publish_id: std::sync::atomic::AtomicU64,
    state: Mutex<BusState<Seq>>,
}

struct BusState<Seq> {
    subscribed_types: HashSet<EventType>,
    last_sequence: Arc<Mutex<Seq>>,
    cancel: Option<CancellationToken>,
    listener_handle: Option<JoinHandle<()>>,
}

impl<Seq, S, L> NotifyingPersistentEventBus<Seq, S, L>
where
    Seq: Sequence,
{
    pub fn new(store: NotifyingEventStore<S, L>) -> Self {
        let (publish_tx, publish_rx) = async_channel::bounded(64);
        Self {
            store: Arc::new(store),
            inmem: Arc::new(InMemEventBus::new()),
            publish_tx,
            publish_rx,
            next_publish_id: std::sync::atomic::AtomicU64::new(0),
            state: Mutex::new(BusState {
                subscribed_types: HashSet::new(),
                last_sequence: Arc::new(Mutex::new(Seq::default())),
                cancel: None,
                listener_handle: None,
            }),
        }
    }

    fn next_id(&self) -> PublishId {
        self.next_publish_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl<Seq, S, L> NotifyingPersistentEventBus<Seq, S, L>
where
    Seq: Sequence,
    S: EventStore<Seq>,
    L: EventListener<Seq>,
{
    async fn ensure_listening<E: DomainEvent>(&self) {
        let event_type = EventType::of::<E>();
        let mut state = self.state.lock().await;

        if state.subscribed_types.contains(&event_type) {
            return;
        }

        state.subscribed_types.insert(event_type);
        self.restart_listener(&mut state).await;
    }

    async fn restart_listener(&self, state: &mut BusState<Seq>) {
        if let Some(cancel) = state.cancel.take() {
            cancel.cancel();
        }
        if let Some(handle) = state.listener_handle.take() {
            if let Err(e) = handle.await {
                error!(error = %e, "Listener task failed during shutdown");
            }
        }

        let last_seq = *state.last_sequence.lock().await;
        let event_types: Vec<EventType> = state.subscribed_types.iter().cloned().collect();
        let store = self.store.clone();
        let inmem = self.inmem.clone();
        let last_sequence = state.last_sequence.clone();
        let publish_rx = self.publish_rx.clone();
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        let handle = tokio::task::Builder::new()
            .name("persistent-event-bus-listener")
            .spawn(Self::listener_task(
                store,
                inmem,
                last_sequence,
                event_types,
                last_seq,
                publish_rx,
                cancel_clone,
            ))
            .expect("failed to spawn listener task");

        state.cancel = Some(cancel);
        state.listener_handle = Some(handle);
    }

    async fn listener_task(
        store: Arc<NotifyingEventStore<S, L>>,
        inmem: Arc<InMemEventBus>,
        last_sequence: Arc<Mutex<Seq>>,
        event_types: Vec<EventType>,
        mut last_seq: Seq,
        publish_rx: async_channel::Receiver<PublishMessage<Seq>>,
        cancel: CancellationToken,
    ) {
        let notifications = match store.listen::<Seq>(event_types.clone()).await {
            Ok(stream) => stream,
            Err(e) => {
                error!(error = %e, "Failed to start event listener");
                return;
            }
        };
        let mut notifications = pin!(notifications);

        // Catch up from last known position
        last_seq = Self::catch_up(&store, &inmem, &last_sequence, &event_types, last_seq).await;

        // Pending events keyed by publish ID
        let mut pending: HashMap<PublishId, SharedEvent> = HashMap::new();

        info!("Persistent event bus listener started");
        loop {
            tokio::select! {
                biased;

                _ = cancel.cancelled() => break,

                msg = publish_rx.recv() => {
                    match msg {
                        Ok(PublishMessage::Pending { id, event }) => {
                            pending.insert(id, event);
                        }
                        Ok(PublishMessage::Committed { id, sequence }) => {
                            if let Some(event) = pending.remove(&id) {
                                last_seq = Self::dispatch_committed(
                                    &store, &inmem, &last_sequence, &event_types,
                                    last_seq, sequence, event,
                                ).await;
                            }
                        }
                        Ok(PublishMessage::Abort { id }) => {
                            pending.remove(&id);
                        }
                        Err(_) => break,
                    }
                }

                notification = notifications.next() => {
                    match notification {
                        Some(_seq) => {
                            last_seq = Self::catch_up(
                                &store, &inmem, &last_sequence, &event_types, last_seq,
                            ).await;
                        }
                        None => break,
                    }
                }
            }
        }
        info!("Persistent event bus listener stopped");
    }

    /// Dispatches a committed local event. If the sequence is contiguous
    /// with `last_seq`, dispatches directly without a DB load. Otherwise
    /// loads the gap first, then dispatches only if the catch-up didn't
    /// already cover this sequence.
    async fn dispatch_committed(
        store: &NotifyingEventStore<S, L>,
        inmem: &InMemEventBus,
        last_sequence: &Mutex<Seq>,
        event_types: &[EventType],
        mut last_seq: Seq,
        sequence: Seq,
        event: SharedEvent,
    ) -> Seq {
        if !last_seq.is_behind(&sequence) {
            // Already dispatched via catch-up
            return last_seq;
        }

        if !last_seq.is_next(&sequence) {
            // Gap — load missing remote events first
            last_seq = Self::catch_up(store, inmem, last_sequence, event_types, last_seq).await;

            if !last_seq.is_behind(&sequence) {
                // Catch-up already loaded this event
                return last_seq;
            }
        }

        // Dispatch the local event directly
        if let Err(e) = inmem.publish_shared(event).await {
            error!(error = %e, "Failed to dispatch committed event");
        }
        last_seq = sequence;
        *last_sequence.lock().await = last_seq;

        last_seq
    }

    async fn catch_up(
        store: &NotifyingEventStore<S, L>,
        inmem: &InMemEventBus,
        last_sequence: &Mutex<Seq>,
        event_types: &[EventType],
        mut last_seq: Seq,
    ) -> Seq {
        let stored_events = match store.load(last_seq, event_types.to_vec(), 100).await {
            Ok(events) => events,
            Err(e) => {
                error!(error = %e, "Failed to load events during catch-up");
                return last_seq;
            }
        };

        for stored in stored_events {
            let shared: SharedEvent = Arc::from(stored.event);
            if let Err(e) = inmem.publish_shared(shared).await {
                error!(error = %e, "Failed to dispatch loaded event");
            }
            if last_seq.is_behind(&stored.sequence) {
                last_seq = stored.sequence;
                *last_sequence.lock().await = last_seq;
            }
        }

        last_seq
    }
}

#[async_trait]
impl<Seq, S, L> EventBus for NotifyingPersistentEventBus<Seq, S, L>
where
    Seq: Sequence,
    S: EventStore<Seq>,
    L: EventListener<Seq>,
{
    async fn publish(&self, event: impl DomainEvent) -> error::Result<()> {
        let event: SharedEvent = Arc::new(event);
        let id = self.next_id();

        // Phase 1: send event to listener before the DB transaction.
        // Send only fails if the listener task is dead (channel closed).
        // The event is still persisted and will be caught up on restart.
        if self
            .publish_tx
            .send(PublishMessage::Pending {
                id,
                event: event.clone(),
            })
            .await
            .is_err()
        {
            error!("Listener task is dead — event will not be dispatched locally");
        }

        // Phase 2: append to store (&*event: &dyn DomainEvent)
        match self.store.append(&*event).await {
            Ok(seq) => {
                if self
                    .publish_tx
                    .send(PublishMessage::Committed { id, sequence: seq })
                    .await
                    .is_err()
                {
                    error!(
                        "Listener task is dead — committed event will not be dispatched locally"
                    );
                }
                Ok(())
            }
            Err(e) => {
                // Best-effort abort — if listener is dead, the pending entry
                // is cleaned up when the task's HashMap is dropped.
                let _ = self.publish_tx.send(PublishMessage::Abort { id }).await;
                Err(e)
            }
        }
    }

    async fn subscribe<E>(&self) -> impl Stream<Item = Arc<E>>
    where
        E: DomainEvent,
    {
        self.ensure_listening::<E>().await;
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
        self.ensure_listening::<E>().await;
        self.inmem
            .start_consumer_with_options(options, consumer)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

    use tokio::time::{timeout, Duration};
    use tokio_stream::wrappers::ReceiverStream;

    use super::*;
    use crate::{
        event::{
            domain_event::fixtures::{StoredTestEvent, TestEvent},
            event_type::EventType,
        },
        persistence::{event_store::fixtures::FailingEventStore, listener::EventListener},
    };

    /// Local mock store that uses tokio::sync::Mutex (async) for compatibility
    /// with the NotifyingPersistentEventBus tests, which require async locking
    /// in the append path (the centralized MockEventStore uses std::sync::Mutex).
    struct MockStore {
        append_count: AtomicUsize,
        next_seq: AtomicI64,
        events: Mutex<Vec<crate::persistence::event_store::StoredEvent<i64>>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                append_count: AtomicUsize::new(0),
                next_seq: AtomicI64::new(1),
                events: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventStore<i64> for MockStore {
        async fn append(&self, event: &dyn DomainEvent) -> error::Result<i64> {
            self.append_count.fetch_add(1, Ordering::SeqCst);
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            let metadata = event.metadata().clone();
            self.events
                .lock()
                .await
                .push(crate::persistence::event_store::StoredEvent {
                    sequence: seq,
                    event: Box::new(StoredTestEvent { metadata }),
                });
            Ok(seq)
        }

        async fn load(
            &self,
            after_sequence: i64,
            _filter: Vec<EventType>,
            limit: usize,
        ) -> error::Result<Vec<crate::persistence::event_store::StoredEvent<i64>>> {
            let events = self.events.lock().await;
            Ok(events
                .iter()
                .filter(|e| e.sequence > after_sequence)
                .take(limit)
                .map(|e| crate::persistence::event_store::StoredEvent {
                    sequence: e.sequence,
                    event: Box::new(StoredTestEvent {
                        metadata: e.event.metadata().clone(),
                    }),
                })
                .collect())
        }
    }

    struct MockListener {
        rx: Mutex<Option<tokio::sync::mpsc::Receiver<i64>>>,
    }

    impl MockListener {
        fn new() -> (Self, tokio::sync::mpsc::Sender<i64>) {
            let (tx, rx) = tokio::sync::mpsc::channel(16);
            (
                Self {
                    rx: Mutex::new(Some(rx)),
                },
                tx,
            )
        }
    }

    #[async_trait]
    impl EventListener<i64> for MockListener {
        async fn listen(
            &self,
            _event_types: Vec<EventType>,
        ) -> error::Result<impl Stream<Item = i64> + Send> {
            let rx = self
                .rx
                .lock()
                .await
                .take()
                .expect("listen called more than once");
            Ok(ReceiverStream::new(rx))
        }
    }

    fn make_bus(
        store: MockStore,
        listener: MockListener,
    ) -> NotifyingPersistentEventBus<i64, MockStore, MockListener> {
        NotifyingPersistentEventBus::new(NotifyingEventStore::new(store, listener))
    }

    #[tokio::test]
    async fn publish_persists_event_in_store() {
        let (listener, _tx) = MockListener::new();
        let bus = make_bus(MockStore::new(), listener);

        bus.publish(TestEvent::new("hello")).await.unwrap();
        bus.publish(TestEvent::new("world")).await.unwrap();

        let events = bus.store.load(0, vec![], 100).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn local_publish_dispatches_directly_without_notification() {
        let (listener, _notify_tx) = MockListener::new();
        let bus = make_bus(MockStore::new(), listener);

        let mut stream = bus.subscribe::<TestEvent>().await;
        tokio::task::yield_now().await; // let listener task start

        bus.publish(TestEvent::new("direct")).await.unwrap();

        let event = timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(event.value, "direct");
    }

    #[tokio::test]
    async fn notification_loads_remote_events() {
        let (listener, notify_tx) = MockListener::new();
        let bus = make_bus(MockStore::new(), listener);

        let mut stream = bus.subscribe::<StoredTestEvent>().await;

        // Simulate a remote publish: insert directly into the store
        bus.store.append(&TestEvent::new("remote")).await.unwrap();

        // Trigger notification
        notify_tx.send(1).await.unwrap();

        let event = timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(event.metadata().expected_version, 0);
    }

    #[tokio::test]
    async fn publish_returns_error_when_store_fails() {
        let (listener, _tx) = MockFailingListener::new();
        let bus: NotifyingPersistentEventBus<i64, _, _> =
            NotifyingPersistentEventBus::new(NotifyingEventStore::new(FailingEventStore, listener));

        let result = bus.publish(TestEvent::new("will fail")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn failed_publish_does_not_dispatch() {
        let (fail_listener, _) = MockFailingListener::new();
        let fail_bus: NotifyingPersistentEventBus<i64, _, _> = NotifyingPersistentEventBus::new(
            NotifyingEventStore::new(FailingEventStore, fail_listener),
        );
        let mut stream = fail_bus.subscribe::<TestEvent>().await;

        let _ = fail_bus.publish(TestEvent::new("should abort")).await;

        let result = timeout(Duration::from_millis(50), stream.next()).await;
        assert!(
            result.is_err(),
            "should have timed out — aborted event must not be dispatched"
        );
    }

    #[tokio::test]
    async fn start_consumer_receives_events() {
        let (listener, _notify_tx) = MockListener::new();
        let bus = make_bus(MockStore::new(), listener);

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let options = SubscriptionOptions::new().named("test-consumer");
        let _handles = bus
            .start_consumer_with_options::<TestEvent, _, _>(options, move |_event| {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            })
            .await
            .unwrap();
        tokio::task::yield_now().await;

        bus.publish(TestEvent::new("a")).await.unwrap();
        bus.publish(TestEvent::new("b")).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn multiple_subscribers_each_receive_event() {
        let (listener, _notify_tx) = MockListener::new();
        let bus = make_bus(MockStore::new(), listener);

        let mut stream1 = bus.subscribe::<TestEvent>().await;
        let mut stream2 = bus.subscribe::<TestEvent>().await;
        tokio::task::yield_now().await;

        bus.publish(TestEvent::new("broadcast")).await.unwrap();

        let e1 = timeout(Duration::from_secs(1), stream1.next())
            .await
            .unwrap()
            .unwrap();
        let e2 = timeout(Duration::from_secs(1), stream2.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(e1.value, "broadcast");
        assert_eq!(e2.value, "broadcast");
    }

    struct MockFailingListener;

    #[async_trait]
    impl EventListener<i64> for MockFailingListener {
        async fn listen(
            &self,
            _event_types: Vec<EventType>,
        ) -> error::Result<impl Stream<Item = i64> + Send> {
            Ok(tokio_stream::empty())
        }
    }

    impl MockFailingListener {
        fn new() -> (Self, ()) {
            (Self, ())
        }
    }
}
