use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use task::domain::{Task, TaskStatus};
use uuid::Uuid;

use crate::api::task::dto::{TaskStatusDto, TaskTypeDto};

impl From<Task> for TaskListResponse {
    fn from(task: Task) -> Self {
        let (status, failed_message) = match task.status {
            TaskStatus::Pending => (TaskStatusDto::Pending, None),
            TaskStatus::InProgress => (TaskStatusDto::InProgress, None),
            TaskStatus::Completed => (TaskStatusDto::Completed, None),
            TaskStatus::Failed(message) => (TaskStatusDto::Failed, Some(message)),
        };
        Self {
            id: task.id,
            task_type: TaskTypeDto::from(task.task_type),
            reference_id: task.reference_id,
            status,
            failed_message,
            created_at: task.created_at.into(),
            started_at: task.started_at.map(DateTime::from),
            completed_at: task.completed_at.map(DateTime::from),
        }
    }
}

/// Response for listing media - optimized for list views with minimal data
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TaskListResponse {
    pub id: Uuid,
    pub task_type: TaskTypeDto,
    pub reference_id: Uuid,
    pub status: TaskStatusDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_message: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<FixedOffset>>,
}
