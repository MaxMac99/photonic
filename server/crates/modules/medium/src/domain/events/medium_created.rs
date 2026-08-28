use derive_new::new;
use kernel::{
    event::{DomainEvent, EventMetadata},
    MediumId, UserId,
};
use serde::{Deserialize, Serialize};

use crate::domain::{MediumItem, MediumType};

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
