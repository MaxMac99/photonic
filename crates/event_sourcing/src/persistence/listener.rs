use crate::error;
use crate::event::event_type::EventType;
use async_trait::async_trait;
use tokio_stream::Stream;

#[async_trait]
pub trait EventListener<Seq>: Send + Sync + 'static {
    /// Returns a stream of sequence numbers for events matching the given types.
    async fn listen(
        &self,
        event_types: Vec<EventType>,
    ) -> error::Result<impl Stream<Item = Seq> + Send>;
}