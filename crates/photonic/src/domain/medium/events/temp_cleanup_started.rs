use derive_new::new;
use uuid::Uuid;

use crate::domain::event::{DomainEvent, EventMetadata};

#[derive(new, Debug, Clone)]
#[new(visibility = "pub(crate)")]
pub struct TempCleanupStartedEvent {
    pub sweep_id: Uuid,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for TempCleanupStartedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}