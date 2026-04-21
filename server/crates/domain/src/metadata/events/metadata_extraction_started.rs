use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::{MediumId, MediumItemId},
    user::UserId,
};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MetadataExtractionStartedEvent {
    pub medium_id: MediumId,
    pub leading_item_id: MediumItemId,
    pub owner_id: UserId,
    #[new(default)]
    pub event_metadata: EventMetadata,
}

impl DomainEvent for MetadataExtractionStartedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.event_metadata
    }
}
