use async_trait::async_trait;
use domain::aggregate::{AggregateRoot, AggregateVersion};

use crate::error::ApplicationResult;

/// Port for loading and appending events for a specific aggregate type.
/// Used by AggregateRepository for replay.
#[async_trait]
pub trait EventStore<A: AggregateRoot>: Send + Sync {
    /// Load all events for an aggregate since a given version.
    /// Pass `None` to load all events from the beginning.
    async fn load_events(
        &self,
        aggregate_id: &str,
        since_version: Option<AggregateVersion>,
    ) -> ApplicationResult<Vec<A::Event>>;

    /// Append new events with optimistic concurrency check.
    /// Fails if the expected version doesn't match the current stream version.
    async fn append_events(
        &self,
        aggregate_id: &str,
        expected_version: AggregateVersion,
        events: Vec<A::Event>,
    ) -> ApplicationResult<()>;
}
