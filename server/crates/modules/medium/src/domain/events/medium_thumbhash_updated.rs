use derive_new::new;
use kernel::{
    event::{DomainEvent, EventMetadata},
    MediumId, UserId,
};
use serde::{Deserialize, Serialize};

use crate::domain::Thumbhash;

/// Emitted when the server has computed the thumbhash for a medium from its
/// tiny thumbnail. The projection denormalizes the hash onto the media row
/// for fast list queries.
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MediumThumbhashUpdatedEvent {
    pub user_id: UserId,
    pub medium_id: MediumId,
    pub thumbhash: Thumbhash,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumThumbhashUpdatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
