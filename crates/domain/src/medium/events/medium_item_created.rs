use derive_new::new;
use mime::Mime;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::{storage::FileLocation, MediumId, MediumItemId, MediumItemType},
    user::UserId,
};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MediumItemCreatedEvent {
    pub user_id: UserId,
    pub medium_id: MediumId,
    pub item_id: MediumItemId,
    pub item_type: MediumItemType,
    pub file_location: FileLocation,
    #[serde(with = "crate::serde_helpers::mime_serde")]
    pub mime_type: Mime,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumItemCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn event_type(&self) -> &'static str {
        "MediumItemCreated"
    }
}