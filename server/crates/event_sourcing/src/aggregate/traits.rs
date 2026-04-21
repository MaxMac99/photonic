use std::any::TypeId;

use crate::event::domain_event::DomainEvent;

/// Marker trait for aggregate roots. Aggregates are reconstituted from
/// event streams and must provide a default (empty) state.
pub trait Aggregate: Default + Send + Sync + 'static {
    /// The type of the aggregate's identity (e.g. `Uuid`, `String`).
    type Id: ToString + Send + Sync;

    /// The aggregate type name used as the stream category (e.g. "Medium").
    fn aggregate_type() -> &'static str;
}

/// A typed aggregate type identifier, similar to [`EventType`](crate::event::event_type::EventType).
#[derive(Debug, Clone)]
pub struct AggregateType {
    id: TypeId,
    name: &'static str,
}

impl AggregateType {
    pub fn of<A: Aggregate>() -> Self {
        Self {
            id: TypeId::of::<A>(),
            name: A::aggregate_type(),
        }
    }

    pub fn id(&self) -> TypeId {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl PartialEq for AggregateType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AggregateType {}

impl std::hash::Hash for AggregateType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Trait for type-safe event application on aggregates.
///
/// Implement this for each event type that belongs to the aggregate's stream.
/// The stream builder enforces at compile time that the aggregate implements
/// `ApplyEvent<E>` for every event type registered with the stream.
pub trait ApplyEvent<E: DomainEvent> {
    fn apply(&mut self, event: &E);
}
