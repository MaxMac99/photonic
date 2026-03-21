mod medium_created;
mod medium_item_created;
mod medium_updated;

pub use medium_created::MediumCreatedEvent;
pub use medium_item_created::MediumItemCreatedEvent;
pub use medium_updated::MediumUpdatedEvent;

use serde::{Deserialize, Serialize};

use crate::event::{DomainEvent, EventMetadata};

/// Sum type of all events for the Medium aggregate.
/// Used by the event store for serialization/deserialization and aggregate replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MediumEvent {
    MediumCreated(MediumCreatedEvent),
    MediumItemCreated(MediumItemCreatedEvent),
    MediumUpdated(MediumUpdatedEvent),
}

impl DomainEvent for MediumEvent {
    fn metadata(&self) -> &EventMetadata {
        match self {
            MediumEvent::MediumCreated(e) => e.metadata(),
            MediumEvent::MediumItemCreated(e) => e.metadata(),
            MediumEvent::MediumUpdated(e) => e.metadata(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            MediumEvent::MediumCreated(e) => e.event_type(),
            MediumEvent::MediumItemCreated(e) => e.event_type(),
            MediumEvent::MediumUpdated(e) => e.event_type(),
        }
    }
}

impl From<MediumCreatedEvent> for MediumEvent {
    fn from(e: MediumCreatedEvent) -> Self {
        MediumEvent::MediumCreated(e)
    }
}

impl From<MediumItemCreatedEvent> for MediumEvent {
    fn from(e: MediumItemCreatedEvent) -> Self {
        MediumEvent::MediumItemCreated(e)
    }
}

impl From<MediumUpdatedEvent> for MediumEvent {
    fn from(e: MediumUpdatedEvent) -> Self {
        MediumEvent::MediumUpdated(e)
    }
}
