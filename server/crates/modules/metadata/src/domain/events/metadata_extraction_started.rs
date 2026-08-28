use derive_new::new;
use kernel::{
    event::{DomainEvent, EventMetadata},
    MediumId, MediumItemId, UserId,
};
use serde::{Deserialize, Serialize};

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
