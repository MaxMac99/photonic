use derive_new::new;
use uuid::Uuid;

use crate::domain::event::{DomainEvent, EventMetadata};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct TempCleanupFailedEvent {
    pub sweep_id: Uuid,
    pub error: String,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for TempCleanupFailedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}