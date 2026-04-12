use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::event::event_type::EventType;
use crate::persistence::event_store::{EventStore, StoredEvent};
use crate::persistence::listener::EventListener;
use async_trait::async_trait;
use tokio_stream::Stream;

/// Event store decorator that composes an inner store with an event listener.
///
/// The inner store handles persistence. The listener provides a stream of
/// sequence numbers signaling that new events (matching a filter) are available.
/// The actual notification mechanism (e.g. Postgres LISTEN/NOTIFY) is external —
/// this struct just exposes the listener's stream alongside the store's operations.
pub struct NotifyingEventStore<S, L> {
    store: S,
    listener: L,
}

impl<S, L> NotifyingEventStore<S, L> {
    pub fn new(store: S, listener: L) -> Self {
        Self { store, listener }
    }
}

#[async_trait]
impl<Seq, S, L> EventStore<Seq> for NotifyingEventStore<S, L>
where
    Seq: Send + 'static,
    S: EventStore<Seq>,
    L: EventListener<Seq>,
{
    async fn append(&self, event: &dyn DomainEvent) -> error::Result<Seq> {
        self.store.append(event).await
    }

    async fn load(
        &self,
        after_sequence: Seq,
        filter: Vec<EventType>,
        limit: usize,
    ) -> error::Result<Vec<StoredEvent<Seq>>> {
        self.store.load(after_sequence, filter, limit).await
    }
}

impl<S, L> NotifyingEventStore<S, L> {
    pub async fn listen<Seq>(
        &self,
        event_types: Vec<EventType>,
    ) -> error::Result<impl Stream<Item = Seq> + Send + use<'_, Seq, S, L>>
    where
        L: EventListener<Seq>,
    {
        self.listener.listen(event_types).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::domain_event::fixtures::TestEvent;
    use crate::persistence::event_store::fixtures::MockEventStore;
    use async_trait::async_trait;
    use futures_util::StreamExt;
    use tokio::time::{timeout, Duration};
    use tokio_stream::wrappers::ReceiverStream;

    /// Listener that produces i64 sequences, matching MockEventStore<i64>.
    struct MockI64Listener {
        rx: tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<i64>>>,
    }

    impl MockI64Listener {
        fn new() -> (Self, tokio::sync::mpsc::Sender<i64>) {
            let (tx, rx) = tokio::sync::mpsc::channel(16);
            (
                Self {
                    rx: tokio::sync::Mutex::new(Some(rx)),
                },
                tx,
            )
        }
    }

    #[async_trait]
    impl EventListener<i64> for MockI64Listener {
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

    #[tokio::test]
    async fn append_delegates_to_inner_store() {
        let (listener, _tx) = MockI64Listener::new();
        let store = NotifyingEventStore::new(MockEventStore::new(), listener);

        store.append(&TestEvent::new("hello")).await.unwrap();
        store.append(&TestEvent::new("world")).await.unwrap();

        assert_eq!(store.store.append_count(), 2);
    }

    #[tokio::test]
    async fn load_delegates_to_inner_store() {
        let (listener, _tx) = MockI64Listener::new();
        let store = NotifyingEventStore::new(MockEventStore::new(), listener);

        let events = store.load(0i64, vec![], 100).await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn listen_returns_stream_from_listener() {
        let (listener, tx) = MockI64Listener::new();
        let store = NotifyingEventStore::new(MockEventStore::new(), listener);

        let mut stream = store.listen::<i64>(vec![]).await.unwrap();

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();

        timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");

        timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
    }

    #[tokio::test]
    async fn listen_passes_event_types_through() {
        let (listener, tx) = MockI64Listener::new();
        let store = NotifyingEventStore::new(MockEventStore::new(), listener);

        let event_types = vec![EventType::of::<TestEvent>()];
        let mut stream = store.listen::<i64>(event_types).await.unwrap();

        tx.send(1).await.unwrap();

        timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
    }
}