use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    sync::Arc,
};

use event_sourcing::{
    error::{EventSourcingError, Result},
    event::domain_event::DomainEvent,
};
use serde::{
    de::{DeserializeOwned, Error as DeError},
    Serialize,
};

type Deserializer = Arc<dyn Fn(&serde_json::Value) -> Result<Box<dyn DomainEvent>> + Send + Sync>;

/// Event metadata extracted during serialization.
pub struct SerializedEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
}

type Serializer = Arc<dyn Fn(&dyn DomainEvent) -> Result<SerializedEvent> + Send + Sync>;

/// Maps event type name strings to deserialization functions and `TypeId`s to
/// serialization functions.
///
/// Uses `type_name::<E>()` as the stable event type identifier stored in the
/// `event_type` column. No custom mapping trait needed — serde bounds are
/// checked at registration time.
pub struct EventTypeRegistry {
    deserializers: HashMap<String, Deserializer>,
    serializers: HashMap<TypeId, Serializer>,
}

impl EventTypeRegistry {
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
            serializers: HashMap::new(),
        }
    }

    /// Register an event type for serialization and deserialization.
    /// The event type name in the database is `type_name::<E>()`.
    pub fn register<E>(&mut self)
    where
        E: DomainEvent + Serialize + DeserializeOwned + 'static,
    {
        let name = type_name::<E>().to_string();

        let deserializer: Deserializer = Arc::new(|payload: &serde_json::Value| {
            let event: E = serde_json::from_value(payload.clone())
                .map_err(|e| EventSourcingError::Deserialization { source: e })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        });
        self.deserializers.insert(name, deserializer);

        let serializer: Serializer = Arc::new(|event: &dyn DomainEvent| {
            let any = event as &dyn std::any::Any;
            let typed =
                any.downcast_ref::<E>()
                    .ok_or_else(|| EventSourcingError::Serialization {
                        source: DeError::custom("type mismatch during serialization"),
                    })?;
            let event_type = type_name::<E>().to_string();
            let payload = serde_json::to_value(typed)
                .map_err(|e| EventSourcingError::Serialization { source: e })?;
            Ok(SerializedEvent {
                event_type,
                payload,
            })
        });
        self.serializers.insert(TypeId::of::<E>(), serializer);
    }

    /// Serialize a domain event. Returns `None` if the event type is not registered.
    pub fn serialize(&self, event: &dyn DomainEvent) -> Option<Result<SerializedEvent>> {
        let type_id = (*event).type_id();
        self.serializers.get(&type_id).map(|s| s(event))
    }

    /// Deserialize a stored event payload given its event_type string.
    pub fn deserialize(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<Box<dyn DomainEvent>> {
        let deserializer = self.deserializers.get(event_type).ok_or_else(|| {
            EventSourcingError::Deserialization {
                source: DeError::custom(format!(
                    "Unknown event type: '{event_type}'. Is it registered in the EventTypeRegistry?"
                )),
            }
        })?;
        deserializer(payload)
    }

    /// Returns all registered event type names.
    pub fn event_types(&self) -> Vec<&str> {
        self.deserializers.keys().map(|s| s.as_str()).collect()
    }
}
