use async_trait::async_trait;

use crate::{
    error,
    event::{domain_event::DomainEvent, event_type::EventType},
};

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
    use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

    use super::*;
    use crate::event::{domain_event::fixtures::StoredTestEvent, event_metadata::EventMetadata};

    /// Mock event store that records appended events with sequence tracking.
    pub struct MockEventStore {
        next_seq: AtomicI64,
        append_count: AtomicUsize,
        events: std::sync::Mutex<Vec<(i64, EventMetadata)>>,
    }

    impl MockEventStore {
        pub fn new() -> Self {
            Self {
                next_seq: AtomicI64::new(1),
                append_count: AtomicUsize::new(0),
                events: std::sync::Mutex::new(Vec::new()),
            }
        }

        pub fn append_count(&self) -> usize {
            self.append_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EventStore<i64> for MockEventStore {
        async fn append(&self, event: &dyn DomainEvent) -> error::Result<i64> {
            self.append_count.fetch_add(1, Ordering::SeqCst);
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.events
                .lock()
                .unwrap()
                .push((seq, event.metadata().clone()));
            Ok(seq)
        }

        async fn load(
            &self,
            after_sequence: i64,
            _filter: Vec<EventType>,
            limit: usize,
        ) -> error::Result<Vec<StoredEvent<i64>>> {
            let events = self.events.lock().unwrap();
            Ok(events
                .iter()
                .filter(|(seq, _)| *seq > after_sequence)
                .take(limit)
                .map(|(seq, metadata)| StoredEvent {
                    sequence: *seq,
                    event: Box::new(StoredTestEvent {
                        metadata: metadata.clone(),
                    }),
                })
                .collect())
        }
    }

    /// Event store that always fails on append. Useful for testing error paths.
    pub struct FailingEventStore;

    #[async_trait]
    impl EventStore<i64> for FailingEventStore {
        async fn append(&self, _event: &dyn DomainEvent) -> error::Result<i64> {
            Err(error::EventSourcingError::Store {
                message: "simulated failure".to_string(),
            })
        }

        async fn load(
            &self,
            _after_sequence: i64,
            _filter: Vec<EventType>,
            _limit: usize,
        ) -> error::Result<Vec<StoredEvent<i64>>> {
            Ok(vec![])
        }
    }
}
