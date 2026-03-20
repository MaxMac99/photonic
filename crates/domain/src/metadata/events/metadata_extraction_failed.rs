use derive_new::new;

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::{MediumId, MediumItemId},
    user::UserId,
};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct MetadataExtractionFailedEvent {
    pub medium_id: MediumId,
    pub leading_item_id: MediumItemId,
    pub owner_id: UserId,
    pub error: String,
    #[new(default)]
    pub event_metadata: EventMetadata,
}

impl DomainEvent for MetadataExtractionFailedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.event_metadata
    }
}
