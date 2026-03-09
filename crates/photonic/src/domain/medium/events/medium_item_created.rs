use derive_new::new;
use mime::Mime;

use crate::domain::{
    event::{DomainEvent, EventMetadata},
    medium::{storage::FileLocation, MediumId, MediumItemId, MediumItemType},
    user::UserId,
};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct MediumItemCreatedEvent {
    pub user_id: UserId,
    pub medium_id: MediumId,
    pub item_id: MediumItemId,
    pub item_type: MediumItemType,
    pub file_location: FileLocation,
    pub mime_type: Mime,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumItemCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
