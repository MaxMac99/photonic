use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::event::event_type::EventType;
use async_trait::async_trait;

pub struct StoredEvent<Seq> {
    pub sequence: Seq,
    pub event: Box<dyn DomainEvent>,
}

#[async_trait]
pub trait EventStore<Seq>: Send + Sync + 'static {
    async fn append(&self, event: &dyn DomainEvent) -> error::Result<Seq>;

    async fn load(
        &self,
        after_sequence: Seq,
        filter: Vec<EventType>,
        limit: usize,
    ) -> error::Result<Vec<StoredEvent<Seq>>>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock event store that records appended events.
    pub struct MockEventStore {
        append_count: AtomicUsize,
    }

    impl MockEventStore {
        pub fn new() -> Self {
            Self {
                append_count: AtomicUsize::new(0),
            }
        }

        pub fn append_count(&self) -> usize {
            self.append_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EventStore<()> for MockEventStore {
        async fn append(&self, _event: &dyn DomainEvent) -> error::Result<()> {
            self.append_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn load(
            &self,
            _after_sequence: (),
            _filter: Vec<EventType>,
            _limit: usize,
        ) -> error::Result<Vec<StoredEvent<()>>> {
            Ok(vec![])
        }
    }

}