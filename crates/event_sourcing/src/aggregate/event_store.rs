use async_trait::async_trait;

use crate::{error, persistence::event_store::StoredEvent, stream::stream_id::StreamId};

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
    /// Load events in a stream after `after_version`, ordered by stream version.
    /// Pass `Seq::default()` to load all events from the beginning.
    async fn load_stream(
        &self,
        stream: &StreamId,
        after_version: Seq,
    ) -> error::Result<Vec<StoredEvent<Seq>>>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::collections::HashMap;

    use super::*;
    use crate::event::domain_event::DomainEvent;

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
        async fn load_stream(
            &self,
            stream: &StreamId,
            after_version: i64,
        ) -> error::Result<Vec<StoredEvent<i64>>> {
            let streams = self.streams.lock().unwrap();
            Ok(streams
                .get(&stream.to_storage_key())
                .map(|entries| {
                    entries
                        .iter()
                        .filter(|(seq, _)| *seq > after_version)
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
