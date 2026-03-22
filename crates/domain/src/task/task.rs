use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use snafu::OptionExt;
use uuid::Uuid;

use super::status::TaskStatus;
use crate::{
    aggregate::{AggregateRoot, AggregateVersion},
    error::{DomainResult, InvariantViolationSnafu},
    task::{
        events::{
            TaskCompletedEvent, TaskCreatedEvent, TaskEvent, TaskFailedEvent, TaskStartedEvent,
        },
        TaskTransition, TaskType,
    },
    user::UserId,
};

pub type TaskId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub task_type: TaskType,
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub version: AggregateVersion,
}

impl AggregateRoot for Task {
    type Event = TaskEvent;

    fn aggregate_type() -> &'static str {
        "Task"
    }

    fn version(&self) -> AggregateVersion {
        self.version
    }

    fn from_initial_event(event: &TaskEvent) -> DomainResult<Self> {
        let TaskEvent::TaskCreated(e) = event else {
            return InvariantViolationSnafu {
                message: "Task aggregate must start with TaskCreated event",
            }
            .fail();
        };
        Ok(Self {
            id: e.task_id,
            task_type: e.task_type,
            reference_id: e.reference_id,
            user_id: e.user_id,
            status: TaskStatus::Pending,
            created_at: e.metadata.occurred_at,
            started_at: None,
            completed_at: None,
            version: 1,
        })
    }

    fn apply(&mut self, event: &TaskEvent) {
        match event {
            TaskEvent::TaskCreated(e) => {
                self.id = e.task_id;
                self.task_type = e.task_type;
                self.reference_id = e.reference_id;
                self.user_id = e.user_id;
                self.status = TaskStatus::Pending;
            }
            TaskEvent::TaskStarted(_) => {
                self.status = TaskStatus::InProgress;
                self.started_at = Some(Utc::now());
            }
            TaskEvent::TaskCompleted(_) => {
                self.status = TaskStatus::Completed;
                self.completed_at = Some(Utc::now());
            }
            TaskEvent::TaskFailed(e) => {
                self.status = TaskStatus::Failed(e.error.clone());
                self.completed_at = Some(Utc::now());
            }
        }
        self.version += 1;
    }
}

/// Namespace for deterministic task ID generation
const TASK_ID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

impl Task {
    /// Creates a new task with a deterministic ID based on (reference_id, task_type, user_id).
    ///
    /// This ensures that multiple handlers creating the same logical task will generate
    /// the same task ID, allowing proper conflict resolution in the database.
    pub fn new(
        task_type: TaskType,
        reference_id: Uuid,
        user_id: UserId,
    ) -> (Self, TaskCreatedEvent) {
        // Generate deterministic ID from the unique business key
        let id = Uuid::new_v5(
            &TASK_ID_NAMESPACE,
            format!("{}:{:?}:{}", reference_id, task_type, user_id).as_bytes(),
        );

        let task = Self {
            id,
            task_type,
            reference_id,
            user_id,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            version: 0,
        };

        let event = TaskCreatedEvent::new(id, task_type, reference_id, user_id);
        (task, event)
    }

    /// Start the task
    /// Business rule: Can only start pending tasks
    pub fn start(&mut self) -> DomainResult<TaskStartedEvent> {
        self.status =
            self.status
                .transition(TaskTransition::Start)
                .context(InvariantViolationSnafu {
                    message: format!("Cannot start task in {:?} status", self.status),
                })?;
        self.started_at = Some(Utc::now());
        Ok(TaskStartedEvent::new(self.id))
    }

    /// Mark task as completed
    /// Business rule: Can only complete in-progress tasks
    pub fn complete(&mut self) -> DomainResult<TaskCompletedEvent> {
        self.status =
            self.status
                .transition(TaskTransition::Complete)
                .context(InvariantViolationSnafu {
                    message: format!("Cannot complete task in {:?} status", self.status),
                })?;
        self.completed_at = Some(Utc::now());
        Ok(TaskCompletedEvent::new(self.id))
    }

    /// Mark task as failed with error message
    /// Business rule: Can fail from any state except Completed
    pub fn fail(&mut self, error: impl Into<String>) -> DomainResult<TaskFailedEvent> {
        let error = error.into();
        self.status = self
            .status
            .transition(TaskTransition::Fail(error.clone()))
            .context(InvariantViolationSnafu {
                message: format!("Cannot fail task in {:?} status", self.status),
            })?;
        self.completed_at = Some(Utc::now());
        Ok(TaskFailedEvent::new(self.id, error))
    }

    pub fn is_retriable(&self) -> bool {
        matches!(self.status, TaskStatus::Failed(_))
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Failed(_))
    }

    /// Duration of task execution (if started and completed)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.signed_duration_since(start)),
            _ => None,
        }
    }
}
