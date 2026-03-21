use async_trait::async_trait;
use domain::aggregate::{AggregateRoot, AggregateVersion};

use crate::error::ApplicationResult;

#[async_trait]
pub trait SnapshotStore<A: AggregateRoot>: Send + Sync {
    /// Load the latest snapshot for an aggregate.
    /// Returns the aggregate state and the version at which the snapshot was taken.
    async fn load_snapshot(
        &self,
        aggregate_id: &str,
    ) -> ApplicationResult<Option<(A, AggregateVersion)>>;

    /// Save a snapshot of the aggregate at its current version.
    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        aggregate: &A,
        version: AggregateVersion,
    ) -> ApplicationResult<()>;
}
