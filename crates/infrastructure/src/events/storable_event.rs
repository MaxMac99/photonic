use domain::{
    medium::events::MediumEvent,
    metadata::events::MetadataEvent,
    task::events::TaskEvent,
    user::events::UserEvent,
};
use serde::{Serialize, de::DeserializeOwned};

/// Infrastructure trait that maps event enum variants to their string type names
/// for the `event_type` column in the event store.
///
/// Compile-time exhaustive: adding a new enum variant without updating the match
/// will cause a compiler error.
pub trait StorableEvent: Serialize + DeserializeOwned + Send + Sync {
    /// Returns the event type name for storage and projection filtering.
    fn event_type_name(&self) -> &'static str;
}

// -- Medium events --

pub struct MediumEventTypes;

impl MediumEventTypes {
    pub const MEDIUM_CREATED: &str = "MediumCreated";
    pub const MEDIUM_ITEM_CREATED: &str = "MediumItemCreated";
    pub const MEDIUM_UPDATED: &str = "MediumUpdated";
}

impl StorableEvent for MediumEvent {
    fn event_type_name(&self) -> &'static str {
        match self {
            MediumEvent::MediumCreated(_) => MediumEventTypes::MEDIUM_CREATED,
            MediumEvent::MediumItemCreated(_) => MediumEventTypes::MEDIUM_ITEM_CREATED,
            MediumEvent::MediumUpdated(_) => MediumEventTypes::MEDIUM_UPDATED,
        }
    }
}

// -- User events --

pub struct UserEventTypes;

impl UserEventTypes {
    pub const USER_CREATED: &str = "UserCreated";
    pub const USER_UPDATED: &str = "UserUpdated";
    pub const QUOTA_RESERVED: &str = "QuotaReserved";
    pub const QUOTA_COMMITTED: &str = "QuotaCommitted";
    pub const QUOTA_RELEASED: &str = "QuotaReleased";
}

impl StorableEvent for UserEvent {
    fn event_type_name(&self) -> &'static str {
        match self {
            UserEvent::UserCreated(_) => UserEventTypes::USER_CREATED,
            UserEvent::UserUpdated(_) => UserEventTypes::USER_UPDATED,
            UserEvent::QuotaReserved(_) => UserEventTypes::QUOTA_RESERVED,
            UserEvent::QuotaCommitted(_) => UserEventTypes::QUOTA_COMMITTED,
            UserEvent::QuotaReleased(_) => UserEventTypes::QUOTA_RELEASED,
        }
    }
}

// -- Metadata events --

pub struct MetadataEventTypes;

impl MetadataEventTypes {
    pub const EXTRACTION_STARTED: &str = "MetadataExtractionStarted";
    pub const EXTRACTED: &str = "MetadataExtracted";
    pub const EXTRACTION_FAILED: &str = "MetadataExtractionFailed";
}

impl StorableEvent for MetadataEvent {
    fn event_type_name(&self) -> &'static str {
        match self {
            MetadataEvent::ExtractionStarted(_) => MetadataEventTypes::EXTRACTION_STARTED,
            MetadataEvent::Extracted(_) => MetadataEventTypes::EXTRACTED,
            MetadataEvent::ExtractionFailed(_) => MetadataEventTypes::EXTRACTION_FAILED,
        }
    }
}

// -- Task events --

pub struct TaskEventTypes;

impl TaskEventTypes {
    pub const TASK_CREATED: &str = "TaskCreated";
    pub const TASK_STARTED: &str = "TaskStarted";
    pub const TASK_COMPLETED: &str = "TaskCompleted";
    pub const TASK_FAILED: &str = "TaskFailed";
}

impl StorableEvent for TaskEvent {
    fn event_type_name(&self) -> &'static str {
        match self {
            TaskEvent::TaskCreated(_) => TaskEventTypes::TASK_CREATED,
            TaskEvent::TaskStarted(_) => TaskEventTypes::TASK_STARTED,
            TaskEvent::TaskCompleted(_) => TaskEventTypes::TASK_COMPLETED,
            TaskEvent::TaskFailed(_) => TaskEventTypes::TASK_FAILED,
        }
    }
}

// -- Helper: deserialize individual events from payload by type name --

/// Deserializes an event from a JSON payload, returning None if the type doesn't
/// match any known event for the given aggregate.
pub fn try_deserialize_event<E: DeserializeOwned>(
    payload: &serde_json::Value,
) -> Result<E, serde_json::Error> {
    serde_json::from_value(payload.clone())
}
