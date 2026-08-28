use kernel::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailedEvent {
    pub task_id: Uuid,
    pub error: String,
    pub metadata: EventMetadata,
}

impl TaskFailedEvent {
    pub(crate) fn new(task_id: Uuid, error: String) -> Self {
        Self {
            task_id,
            error,
            metadata: EventMetadata::default(),
        }
    }
}

impl DomainEvent for TaskFailedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
