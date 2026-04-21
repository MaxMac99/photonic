use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    medium::{MediumId, MediumItem, MediumType},
    user::UserId,
};

/// Event emitted when a new medium is created, including its initial item.
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MediumCreatedEvent {
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub medium_type: MediumType,
    pub initial_item: MediumItem,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
