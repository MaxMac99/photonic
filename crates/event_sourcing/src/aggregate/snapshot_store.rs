use crate::aggregate::traits::Aggregate;
use crate::error;
use crate::persistence::sequence::Sequence;
use crate::stream::stream_id::StreamId;
use async_trait::async_trait;

/// A point-in-time snapshot of an aggregate's state.
pub struct Snapshot<A, Seq> {
    pub state: A,
    pub version: Seq,
}

/// Loads and saves aggregate snapshots. Snapshots accelerate aggregate
/// loading by providing a starting state so only events after the snapshot
/// need to be replayed.
#[async_trait]
pub trait SnapshotStore<A: Aggregate, Seq: Sequence>: Send + Sync + 'static {
    /// Load the latest snapshot for a stream. Returns `None` if no snapshot exists.
    async fn load(&self, stream: &StreamId) -> error::Result<Option<Snapshot<A, Seq>>>;

    /// Save a snapshot of the current aggregate state at the given version.
    async fn save(&self, stream: &StreamId, snapshot: &Snapshot<A, Seq>) -> error::Result<()>;
}
