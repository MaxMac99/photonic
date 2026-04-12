use crate::error;
use crate::event::stream::StreamId;
use async_trait::async_trait;

/// Writes stream link records within a transaction.
///
/// When an event is published, the [`StreamLinkingProjection`](crate::projection::stream_linking::StreamLinkingProjection)
/// determines which streams the event belongs to and calls this store to
/// persist the links. The implementation assigns per-stream versions
/// internally (e.g. via `MAX(version) + 1` or a database sequence).
#[async_trait]
pub trait StreamLinkStore<Seq, Tx>: Send + Sync + 'static {
    /// Link an event (by global sequence) to a stream within a transaction.
    async fn link(
        &self,
        sequence: Seq,
        stream: &StreamId,
        tx: &mut Tx,
    ) -> error::Result<()>;
}
