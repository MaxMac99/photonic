use std::sync::Arc;

use domain::aggregate::AggregateRoot;

use crate::{
    error::{ApplicationError, ApplicationResult},
    event_store::EventStore,
    snapshot_store::SnapshotStore,
};

/// Loads aggregates by replaying events from the event store.
/// Optionally uses snapshots as a starting point to reduce replay time.
pub struct AggregateRepository<A: AggregateRoot> {
    event_store: Arc<dyn EventStore<A>>,
    snapshot_store: Option<Arc<dyn SnapshotStore<A>>>,
}

impl<A: AggregateRoot> AggregateRepository<A> {
    pub fn new(
        event_store: Arc<dyn EventStore<A>>,
        snapshot_store: Option<Arc<dyn SnapshotStore<A>>>,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
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
}
