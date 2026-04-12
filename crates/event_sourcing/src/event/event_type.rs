use crate::event::domain_event::DomainEvent;
use std::any::{type_name, TypeId};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct EventType {
    id: TypeId,
    name: String,
}

impl PartialEq for EventType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for EventType {}

impl Hash for EventType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl EventType {
    pub fn of<T: DomainEvent + 'static>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            name: type_name::<T>().to_string(),
        }
    }

    /// Construct an `EventType` from a `&dyn DomainEvent`. Uses the concrete
    /// type's `TypeId` for equality/hashing. The `name` field is a generic
    /// `"dyn DomainEvent"` placeholder since the concrete name is not available.
    pub fn of_dyn(event: &dyn DomainEvent) -> Self {
        Self {
            id: (*event).type_id(),
            name: type_name::<dyn DomainEvent>().to_string(),
        }
    }

    /// Construct from a raw `TypeId`. The name is a placeholder since
    /// the concrete type name is not available from a `TypeId` alone.
    pub fn from_type_id(id: TypeId) -> Self {
        Self {
            id,
            name: "<unknown>".to_string(),
        }
    }

    pub fn id(&self) -> TypeId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
