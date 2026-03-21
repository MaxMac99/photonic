use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::{MediumId, MediumItemId},
    metadata::metadata::Metadata,
    user::UserId,
};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MetadataExtractedEvent {
    pub medium_id: MediumId,
    pub leading_item_id: MediumItemId,
    pub owner_id: UserId,
    pub metadata: Metadata,
    #[new(default)]
    pub event_metadata: EventMetadata,
}

impl DomainEvent for MetadataExtractedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.event_metadata
    }

    fn event_type(&self) -> &'static str {
        "MetadataExtracted"
    }
}
