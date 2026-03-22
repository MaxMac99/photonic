use domain::{
    event::DomainEvent,
    medium::events::MediumEvent,
    metadata::events::MetadataEvent,
    task::events::TaskEvent,
    user::events::UserEvent,
};
use serde::{Serialize, de::DeserializeOwned};

/// Infrastructure trait that maps event enum variants to their string type names
/// for the `event_type` column in the event store.
///
/// Extends `DomainEvent` — storable events are domain events with serialization
/// and type name metadata for persistence.
///
/// Compile-time exhaustive: adding a new enum variant without updating the match
/// will cause a compiler error.
pub trait StorableEvent: DomainEvent + Serialize + DeserializeOwned {
    /// The aggregate type name (e.g., "Medium", "User").
    fn aggregate_type() -> &'static str;

    /// The aggregate instance ID this event belongs to.
    fn aggregate_id(&self) -> String;

    /// Returns the event type name for storage and projection filtering.
    fn event_type_name(&self) -> &'static str;

    /// Returns all event type names for this aggregate's events.
    /// Used by the projection engine to filter which events to fetch.
    fn all_event_types() -> &'static [&'static str];
}

// -- Medium events --

pub struct MediumEventTypes;

impl MediumEventTypes {
    pub const MEDIUM_CREATED: &str = "MediumCreated";
    pub const MEDIUM_ITEM_CREATED: &str = "MediumItemCreated";
    pub const MEDIUM_UPDATED: &str = "MediumUpdated";
}

impl StorableEvent for MediumEvent {
    fn aggregate_type() -> &'static str {
        "Medium"
    }

    fn aggregate_id(&self) -> String {
        match self {
            MediumEvent::MediumCreated(e) => e.medium_id.to_string(),
            MediumEvent::MediumItemCreated(e) => e.medium_id.to_string(),
            MediumEvent::MediumUpdated(e) => e.medium_id.to_string(),
        }
    }

    fn event_type_name(&self) -> &'static str {
        match self {
            MediumEvent::MediumCreated(_) => MediumEventTypes::MEDIUM_CREATED,
            MediumEvent::MediumItemCreated(_) => MediumEventTypes::MEDIUM_ITEM_CREATED,
            MediumEvent::MediumUpdated(_) => MediumEventTypes::MEDIUM_UPDATED,
        }
    }

    fn all_event_types() -> &'static [&'static str] {
        &[
            MediumEventTypes::MEDIUM_CREATED,
            MediumEventTypes::MEDIUM_ITEM_CREATED,
            MediumEventTypes::MEDIUM_UPDATED,
        ]
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
    fn aggregate_type() -> &'static str {
        "User"
    }

    fn aggregate_id(&self) -> String {
        match self {
            UserEvent::UserCreated(e) => e.user_id.to_string(),
            UserEvent::UserUpdated(e) => e.user_id.to_string(),
            UserEvent::QuotaReserved(e) => e.user_id.to_string(),
            UserEvent::QuotaCommitted(e) => e.user_id.to_string(),
            UserEvent::QuotaReleased(e) => e.user_id.to_string(),
        }
    }

    fn event_type_name(&self) -> &'static str {
        match self {
            UserEvent::UserCreated(_) => UserEventTypes::USER_CREATED,
            UserEvent::UserUpdated(_) => UserEventTypes::USER_UPDATED,
            UserEvent::QuotaReserved(_) => UserEventTypes::QUOTA_RESERVED,
            UserEvent::QuotaCommitted(_) => UserEventTypes::QUOTA_COMMITTED,
            UserEvent::QuotaReleased(_) => UserEventTypes::QUOTA_RELEASED,
        }
    }

    fn all_event_types() -> &'static [&'static str] {
        &[
            UserEventTypes::USER_CREATED,
            UserEventTypes::USER_UPDATED,
            UserEventTypes::QUOTA_RESERVED,
            UserEventTypes::QUOTA_COMMITTED,
            UserEventTypes::QUOTA_RELEASED,
        ]
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
    fn aggregate_type() -> &'static str {
        "Metadata"
    }

    fn aggregate_id(&self) -> String {
        match self {
            MetadataEvent::ExtractionStarted(e) => e.medium_id.to_string(),
            MetadataEvent::Extracted(e) => e.medium_id.to_string(),
            MetadataEvent::ExtractionFailed(e) => e.medium_id.to_string(),
        }
    }

    fn event_type_name(&self) -> &'static str {
        match self {
            MetadataEvent::ExtractionStarted(_) => MetadataEventTypes::EXTRACTION_STARTED,
            MetadataEvent::Extracted(_) => MetadataEventTypes::EXTRACTED,
            MetadataEvent::ExtractionFailed(_) => MetadataEventTypes::EXTRACTION_FAILED,
        }
    }

    fn all_event_types() -> &'static [&'static str] {
        &[
            MetadataEventTypes::EXTRACTION_STARTED,
            MetadataEventTypes::EXTRACTED,
            MetadataEventTypes::EXTRACTION_FAILED,
        ]
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
    fn aggregate_type() -> &'static str {
        "Task"
    }

    fn aggregate_id(&self) -> String {
        match self {
            TaskEvent::TaskCreated(e) => e.task_id.to_string(),
            TaskEvent::TaskStarted(e) => e.task_id.to_string(),
            TaskEvent::TaskCompleted(e) => e.task_id.to_string(),
            TaskEvent::TaskFailed(e) => e.task_id.to_string(),
        }
    }

    fn event_type_name(&self) -> &'static str {
        match self {
            TaskEvent::TaskCreated(_) => TaskEventTypes::TASK_CREATED,
            TaskEvent::TaskStarted(_) => TaskEventTypes::TASK_STARTED,
            TaskEvent::TaskCompleted(_) => TaskEventTypes::TASK_COMPLETED,
            TaskEvent::TaskFailed(_) => TaskEventTypes::TASK_FAILED,
        }
    }

    fn all_event_types() -> &'static [&'static str] {
        &[
            TaskEventTypes::TASK_CREATED,
            TaskEventTypes::TASK_STARTED,
            TaskEventTypes::TASK_COMPLETED,
            TaskEventTypes::TASK_FAILED,
        ]
    }
}

