use std::sync::Arc;

use domain::aggregate::{AggregateRoot, AggregateVersion};

use crate::{
    error::ApplicationResult,
    event_store::EventStore,
    snapshot_store::SnapshotStore,
};

/// Generic repository that loads aggregates from events (with optional snapshots)
/// and persists new events to the event store.
pub struct AggregateRepository<A: AggregateRoot> {
    event_store: Arc<dyn EventStore<A>>,
    snapshot_store: Option<Arc<dyn SnapshotStore<A>>>,
    snapshot_interval: AggregateVersion,
}

impl<A: AggregateRoot + Default> AggregateRepository<A> {
    pub fn new(
        event_store: Arc<dyn EventStore<A>>,
        snapshot_store: Option<Arc<dyn SnapshotStore<A>>>,
        snapshot_interval: AggregateVersion,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            snapshot_interval,
        }
    }

    /// Load an aggregate by replaying its events.
    /// Uses snapshot as starting point if available.
    pub async fn load(&self, aggregate_id: &str) -> ApplicationResult<Option<A>> {
        let (mut aggregate, since_version) = match &self.snapshot_store {
            Some(store) => match store.load_snapshot(aggregate_id).await? {
                Some((snapshot, version)) => (snapshot, Some(version)),
                None => (A::default(), None),
            },
            None => (A::default(), None),
        };

        let events = self
            .event_store
            .load_events(aggregate_id, since_version)
            .await?;

        if events.is_empty() && since_version.is_none() {
            return Ok(None);
        }

        for event in &events {
            aggregate.apply(event);
        }

        Ok(Some(aggregate))
    }

    /// Save new events for an aggregate.
    /// Uses optimistic concurrency: expected_version is the aggregate's version
    /// before the new events were applied.
    pub async fn save(
        &self,
        aggregate_id: &str,
        aggregate: &A,
        new_events: Vec<A::Event>,
    ) -> ApplicationResult<()> {
        let expected_version = aggregate.version() - new_events.len() as AggregateVersion;

        self.event_store
            .append_events(aggregate_id, expected_version, new_events)
            .await?;

        // Save snapshot if we've crossed a snapshot interval boundary
        if self.snapshot_interval > 0 {
            if let Some(ref snapshot_store) = self.snapshot_store {
                let prev_snapshot_boundary = expected_version / self.snapshot_interval;
                let curr_snapshot_boundary = aggregate.version() / self.snapshot_interval;
                if curr_snapshot_boundary > prev_snapshot_boundary {
                    snapshot_store
                        .save_snapshot(aggregate_id, aggregate, aggregate.version())
                        .await?;
                }
            }
        }

        Ok(())
    }
}
