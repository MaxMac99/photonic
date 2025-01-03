use chrono::{DateTime, FixedOffset};
use derive_builder::Builder;
use mime_serde_shim::Wrapper as Mime;
use photonic_derive::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, Event)]
#[event(topic = "MediumItemExifLoaded")]
pub struct MediumItemExifLoadedEvent {
    #[event(key)]
    pub id: Uuid,
    pub date: Option<DateTime<FixedOffset>>,
    pub mime: Option<Mime>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
}
