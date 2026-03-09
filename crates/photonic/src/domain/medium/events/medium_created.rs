use derive_new::new;

use crate::domain::{
    event::{DomainEvent, EventMetadata},
    medium::{storage::FileLocation, MediumId, MediumItemId, MediumType},
    user::UserId,
};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct MediumCreatedEvent {
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub medium_type: MediumType,
    pub leading_item_id: MediumItemId,
    pub leading_item_location: FileLocation,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
