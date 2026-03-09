use chrono::{DateTime, Utc};
use snafu::OptionExt;
use uuid::Uuid;

use super::status::TaskStatus;
use crate::domain::task::types::TaskType;
use crate::domain::{
    error::{DomainResult, InvariantViolationSnafu},
    task::TaskTransition,
    user::UserId,
};

pub type TaskId = Uuid;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub task_type: TaskType,
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
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
    pub fn new(task_type: TaskType, reference_id: Uuid, user_id: UserId) -> Self {
        // Generate deterministic ID from the unique business key
        let id = Uuid::new_v5(
            &TASK_ID_NAMESPACE,
            format!("{}:{:?}:{}", reference_id, task_type, user_id).as_bytes(),
        );

        Self {
            id,
            task_type,
            reference_id,
            user_id,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Start the task
    /// Business rule: Can only start pending tasks
    pub fn start(&mut self) -> DomainResult<()> {
        self.status =
            self.status
                .transition(TaskTransition::Start)
                .context(InvariantViolationSnafu {
                    message: format!("Cannot start task in {:?} status", self.status),
                })?;
        self.started_at = Some(Utc::now());
        Ok(())
    }

    /// Mark task as completed
    /// Business rule: Can only complete in-progress tasks
    pub fn complete(&mut self) -> DomainResult<()> {
        self.status =
            self.status
                .transition(TaskTransition::Complete)
                .context(InvariantViolationSnafu {
                    message: format!("Cannot complete task in {:?} status", self.status),
                })?;
        self.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Mark task as failed with error message
    /// Business rule: Can fail from any state except Completed
    pub fn fail(&mut self, error: impl Into<String>) -> DomainResult<()> {
        self.status = self
            .status
            .transition(TaskTransition::Fail(error.into()))
            .context(InvariantViolationSnafu {
                message: format!("Cannot fail task in {:?} status", self.status),
            })?;
        self.completed_at = Some(Utc::now());
        Ok(())
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
