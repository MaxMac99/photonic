use kernel::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedEvent {
    pub task_id: Uuid,
    pub metadata: EventMetadata,
}

impl TaskCompletedEvent {
    pub(crate) fn new(task_id: Uuid) -> Self {
        Self {
            task_id,
            metadata: EventMetadata::default(),
        }
    }
}

impl DomainEvent for TaskCompletedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
