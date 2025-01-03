use crate::storage::StorageLocation;
use derive_builder::Builder;
use photonic_derive::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, Event)]
#[event(topic = "MediumItemMoved")]
pub struct MediumItemMovedEvent {
    #[event(key)]
    pub id: Uuid,
    pub new_location: StorageLocation,
}
