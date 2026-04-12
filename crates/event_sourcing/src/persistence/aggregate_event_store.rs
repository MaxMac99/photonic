use crate::error;
use crate::event::stream::StreamId;
use crate::persistence::event_store::StoredEvent;
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
