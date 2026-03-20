use derive_new::new;

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::MediumId,
    user::UserId,
};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct MediumUpdatedEvent {
    pub medium_id: MediumId,
    pub owner_id: UserId,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumUpdatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
