use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    event::{DomainEvent, EventMetadata},
    task::TaskType,
    user::UserId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreatedEvent {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub metadata: EventMetadata,
}

impl TaskCreatedEvent {
    pub(crate) fn new(
        task_id: Uuid,
        task_type: TaskType,
        reference_id: Uuid,
        user_id: UserId,
    ) -> Self {
        Self {
            task_id,
            task_type,
            reference_id,
            user_id,
            metadata: EventMetadata::default(),
        }
    }
}

impl DomainEvent for TaskCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
