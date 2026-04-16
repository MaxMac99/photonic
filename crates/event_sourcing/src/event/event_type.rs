use std::{
    any::{type_name, TypeId},
    hash::{Hash, Hasher},
};

use crate::event::domain_event::DomainEvent;

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

    pub fn id(&self) -> TypeId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
