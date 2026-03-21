use domain::event::{DomainEvent, EventMetadata};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TempCleanupStartedEvent {
    pub sweep_id: Uuid,
    pub metadata: EventMetadata,
}

impl TempCleanupStartedEvent {
    pub fn new(sweep_id: Uuid) -> Self {
        Self {
            sweep_id,
            metadata: EventMetadata::default(),
        }
    }
}

impl DomainEvent for TempCleanupStartedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn event_type(&self) -> &'static str {
        "TempCleanupStarted"
    }
}

#[derive(Debug, Clone)]
pub struct TempCleanupCompletedEvent {
    pub sweep_id: Uuid,
    pub items_cleaned: usize,
    pub metadata: EventMetadata,
}

impl TempCleanupCompletedEvent {
    pub fn new(sweep_id: Uuid, items_cleaned: usize) -> Self {
        Self {
            sweep_id,
            items_cleaned,
            metadata: EventMetadata::default(),
        }
    }
}

impl DomainEvent for TempCleanupCompletedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn event_type(&self) -> &'static str {
        "TempCleanupCompleted"
    }
}

#[derive(Debug, Clone)]
pub struct TempCleanupFailedEvent {
    pub sweep_id: Uuid,
    pub error: String,
    pub metadata: EventMetadata,
}

impl TempCleanupFailedEvent {
    pub fn new(sweep_id: Uuid, error: String) -> Self {
        Self {
            sweep_id,
            error,
            metadata: EventMetadata::default(),
        }
    }
}

impl DomainEvent for TempCleanupFailedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn event_type(&self) -> &'static str {
        "TempCleanupFailed"
    }
}
