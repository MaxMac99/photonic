use crate::medium::MediumType;
use chrono::{DateTime, FixedOffset, Utc};
use derive_builder::Builder;
use photonic_derive::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, Event)]
#[event(topic = "MediumUpdated")]
pub struct MediumUpdatedEvent {
    #[event(key)]
    pub id: Uuid,
    pub medium_type: MediumType,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub date_added: DateTime<Utc>,
    pub tags: Vec<String>,
}
