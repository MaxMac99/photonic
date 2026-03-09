use derive_new::new;
use uuid::Uuid;

use crate::domain::event::{DomainEvent, EventMetadata};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct TempCleanupCompletedEvent {
    pub sweep_id: Uuid,
    pub items_cleaned: usize,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for TempCleanupCompletedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}