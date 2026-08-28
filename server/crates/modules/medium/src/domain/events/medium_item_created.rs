use byte_unit::Byte;
use derive_new::new;
use kernel::{
    event::{DomainEvent, EventMetadata},
    Dimensions, FileLocation, Filename, MediumId, MediumItemId, Priority, UserId,
};
use mime::Mime;
use serde::{Deserialize, Serialize};

use crate::domain::MediumItemType;

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct MediumItemCreatedEvent {
    pub user_id: UserId,
    pub medium_id: MediumId,
    pub item_id: MediumItemId,
    pub item_type: MediumItemType,
    pub file_location: FileLocation,
    #[serde(with = "kernel::serde_helpers::mime_serde")]
    pub mime_type: Mime,
    pub filename: Filename,
    pub filesize: Byte,
    pub priority: Priority,
    pub dimensions: Option<Dimensions>,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for MediumItemCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
