use crate::{medium::MediumItemType, storage::StorageLocation};
use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, Utc};
use derive_builder::Builder;
use mime_serde_shim::Wrapper as Mime;
use photonic_derive::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, Event)]
#[event(topic = "MediumCreated")]
pub struct MediumCreatedEvent {
    #[event(key)]
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub location: StorageLocation,
    pub size: Byte,
    pub mime: Mime,
    pub filename: String,
    pub extension: String,
    pub user: Uuid,
    pub priority: i32,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_added: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, Event)]
#[event(topic = "MediumItemCreated")]
pub struct MediumItemCreatedEvent {
    #[event(key)]
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub location: StorageLocation,
    pub size: Byte,
    pub mime: Mime,
    pub filename: String,
    pub extension: String,
    pub user: Uuid,
    pub priority: i32,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_added: DateTime<Utc>,
}
