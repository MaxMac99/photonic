use std::collections::HashMap;
use std::sync::Arc;

use event_sourcing::error::{EventSourcingError, Result};
use event_sourcing::event::domain_event::DomainEvent;
use serde::de::DeserializeOwned;

use super::storable_event::StorableEvent;

type Deserializer = Arc<dyn Fn(&serde_json::Value) -> Result<Box<dyn DomainEvent>> + Send + Sync>;

/// Maps event_type strings to deserialization functions. Built from
/// `StorableEvent` implementations at startup.
pub struct EventTypeRegistry {
    deserializers: HashMap<String, Deserializer>,
}

impl EventTypeRegistry {
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
        }
    }

    /// Register all event types from a `StorableEvent` implementation.
    /// Each event type name maps to a deserializer that produces the
    /// concrete event type as `Box<dyn DomainEvent>`.
    pub fn register<E>(&mut self)
    where
        E: StorableEvent + DeserializeOwned + 'static,
    {
        for event_type in E::all_event_types() {
            let deserializer: Deserializer = Arc::new(|payload: &serde_json::Value| {
                let event: E = serde_json::from_value(payload.clone()).map_err(|e| {
                    EventSourcingError::Deserialization { source: e }
                })?;
                Ok(Box::new(event) as Box<dyn DomainEvent>)
            });
            self.deserializers.insert(event_type.to_string(), deserializer);
        }
    }

    /// Deserialize a stored event payload given its event_type string.
    pub fn deserialize(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<Box<dyn DomainEvent>> {
        let deserializer = self.deserializers.get(event_type).ok_or_else(|| {
            EventSourcingError::Deserialization {
                source: serde_json::Error::custom(format!(
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
