mod task_completed;
mod task_created;
mod task_failed;
mod task_started;

pub use task_completed::TaskCompletedEvent;
pub use task_created::TaskCreatedEvent;
pub use task_failed::TaskFailedEvent;
pub use task_started::TaskStartedEvent;

use serde::{Deserialize, Serialize};

use crate::event::{DomainEvent, EventMetadata};

/// Sum type of all events for the Task aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskEvent {
    TaskCreated(TaskCreatedEvent),
    TaskStarted(TaskStartedEvent),
    TaskCompleted(TaskCompletedEvent),
    TaskFailed(TaskFailedEvent),
}

impl DomainEvent for TaskEvent {
    fn metadata(&self) -> &EventMetadata {
        match self {
            TaskEvent::TaskCreated(e) => e.metadata(),
            TaskEvent::TaskStarted(e) => e.metadata(),
            TaskEvent::TaskCompleted(e) => e.metadata(),
            TaskEvent::TaskFailed(e) => e.metadata(),
        }
    }
}

impl From<TaskCreatedEvent> for TaskEvent {
    fn from(e: TaskCreatedEvent) -> Self {
        TaskEvent::TaskCreated(e)
    }
}

impl From<TaskStartedEvent> for TaskEvent {
    fn from(e: TaskStartedEvent) -> Self {
        TaskEvent::TaskStarted(e)
    }
}

impl From<TaskCompletedEvent> for TaskEvent {
    fn from(e: TaskCompletedEvent) -> Self {
        TaskEvent::TaskCompleted(e)
    }
}

impl From<TaskFailedEvent> for TaskEvent {
    fn from(e: TaskFailedEvent) -> Self {
        TaskEvent::TaskFailed(e)
    }
}
