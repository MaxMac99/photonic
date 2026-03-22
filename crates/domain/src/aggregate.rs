use serde::{de::DeserializeOwned, Serialize};

use crate::{error::DomainResult, event::DomainEvent};

pub type AggregateVersion = i64;

/// Core event sourcing trait for aggregate roots.
/// Aggregates are reconstituted by replaying events through `apply`.
pub trait AggregateRoot: Send + Sync + Sized {
    type Event: DomainEvent + Serialize + DeserializeOwned;

    /// Unique aggregate type name (used as stream prefix in event store)
    fn aggregate_type() -> &'static str;

    /// Current version (number of events applied)
    fn version(&self) -> AggregateVersion;

    /// Create the aggregate from its first event (the creation event).
    /// Used during reconstitution when no snapshot exists.
    /// Returns an error if the event is not a valid creation event.
    fn from_initial_event(event: &Self::Event) -> DomainResult<Self>;

    /// Apply a domain event to mutate aggregate state.
    /// Called both during reconstitution (replay) and after command execution.
    /// Must be infallible — events represent facts that already happened.
    fn apply(&mut self, event: &Self::Event);
}