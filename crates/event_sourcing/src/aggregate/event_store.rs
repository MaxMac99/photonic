use crate::error;
use crate::persistence::event_store::StoredEvent;
use crate::stream::stream_id::StreamId;
use async_trait::async_trait;

/// Store for loading aggregate event streams.
///
/// Separate from [`EventStore`](crate::persistence::event_store::EventStore)
/// which handles global append/load for the event bus. The same infrastructure
/// struct can implement both traits against the same storage.
///
/// Writing is handled by the event bus via `EventStore::append`. Stream
/// linking (associating events with streams) happens automatically during
/// publish.
#[async_trait]
pub trait AggregateEventStore<Seq>: Send + Sync + 'static {
    /// Load all events in a stream, ordered by stream version.
    async fn load_stream(&self, stream: &StreamId) -> error::Result<Vec<StoredEvent<Seq>>>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use crate::event::domain_event::DomainEvent;
    use std::collections::HashMap;

    type EventFactory = Box<dyn Fn() -> Box<dyn DomainEvent> + Send + Sync>;

    pub struct MockAggregateEventStore {
        streams: std::sync::Mutex<HashMap<String, Vec<(i64, EventFactory)>>>,
    }

    impl MockAggregateEventStore {
        pub fn new() -> Self {
            Self {
                streams: std::sync::Mutex::new(HashMap::new()),
            }
        }

        pub fn add_event(&self, stream_key: &str, seq: i64, factory: EventFactory) {
            self.streams
                .lock()
                .unwrap()
                .entry(stream_key.to_string())
                .or_default()
                .push((seq, factory));
        }
    }

    #[async_trait]
    impl AggregateEventStore<i64> for MockAggregateEventStore {
        async fn load_stream(&self, stream: &StreamId) -> error::Result<Vec<StoredEvent<i64>>> {
            let streams = self.streams.lock().unwrap();
            Ok(streams
                .get(&stream.to_storage_key())
                .map(|entries| {
                    entries
                        .iter()
                        .map(|(seq, factory)| StoredEvent {
                            sequence: *seq,
                            event: factory(),
                        })
                        .collect()
                })
                .unwrap_or_default())
        }
    }
}
