use chrono::NaiveDateTime;
use domain::task::{Task, TaskStatus};
use uuid::Uuid;

use crate::persistence::postgres::task::task_types::{TaskStatusDb, TaskTypeDb};

#[derive(Debug, sqlx::FromRow)]
pub struct TaskDb {
    pub id: Uuid,
    pub reference_id: Uuid,
    pub user_id: Uuid,
    pub task_type: TaskTypeDb,
    pub status: TaskStatusDb,
    pub error: Option<String>,
    pub created_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

impl From<TaskDb> for Task {
    fn from(val: TaskDb) -> Self {
        let status = match val.status {
            TaskStatusDb::Pending => TaskStatus::Pending,
            TaskStatusDb::InProgress => TaskStatus::InProgress,
            TaskStatusDb::Completed => TaskStatus::Completed,
            TaskStatusDb::Failed => TaskStatus::Failed(val.error.unwrap_or("".to_string())),
        };
        Task {
            id: val.id,
            reference_id: val.reference_id,
            user_id: val.user_id,
            task_type: val.task_type.into(),
            status,
            created_at: val.created_at.and_utc(),
            started_at: val.started_at.map(|d| d.and_utc()),
            completed_at: val.completed_at.map(|d| d.and_utc()),
            version: 0,
        }
    }
}

impl From<Task> for TaskDb {
    fn from(task: Task) -> Self {
        let error = match task.status {
            TaskStatus::Failed(ref err) => Some(err.clone()),
            _ => None,
        };
        Self {
            id: task.id,
            user_id: task.user_id,
            task_type: task.task_type.into(),
            reference_id: task.reference_id,
            status: task.status.into(),
            error,
            created_at: task.created_at.naive_utc(),
            started_at: task.started_at.map(|d| d.naive_utc()),
            completed_at: task.completed_at.map(|d| d.naive_utc()),
        }
    }
}
