use std::sync::Arc;

use domain::aggregate::{AggregateRoot, AggregateVersion};

use crate::{
    error::{ApplicationError, ApplicationResult},
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

impl<A: AggregateRoot> AggregateRepository<A> {
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
    /// Returns None if no events exist for this aggregate.
    pub async fn load(&self, aggregate_id: &str) -> ApplicationResult<Option<A>> {
        // Try loading from snapshot first
        let (mut aggregate, since_version) = match &self.snapshot_store {
            Some(store) => match store.load_snapshot(aggregate_id).await? {
                Some((snapshot, version)) => (Some(snapshot), Some(version)),
                None => (None, None),
            },
            None => (None, None),
        };

        // Load events since snapshot (or all events if no snapshot)
        let events = self
            .event_store
            .load_events(aggregate_id, since_version)
            .await?;

        if events.is_empty() && aggregate.is_none() {
            return Ok(None);
        }

        // Replay events onto the aggregate
        for event in &events {
            match &mut aggregate {
                Some(agg) => agg.apply(event),
                None => {
                    aggregate = Some(
                        A::from_initial_event(event)
                            .map_err(|e| ApplicationError::Domain { source: e })?,
                    );
                }
            }
        }

        Ok(aggregate)
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
