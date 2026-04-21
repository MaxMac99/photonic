use chrono::{DateTime, FixedOffset};
use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::{camera::GpsCoordinates, MediumId},
    user::UserId,
};

/// Event published when a medium's denormalized metadata is updated.
/// Carries the full state change delta so the aggregate can be reconstructed from events.
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MediumUpdatedEvent {
    pub medium_id: MediumId,
    pub owner_id: UserId,
    pub taken_at: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_coordinates: Option<GpsCoordinates>,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumUpdatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
