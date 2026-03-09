use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{status::TaskStatus, task::TaskId, TaskType};
use crate::shared::value_objects::{KeysetCursor, SortDirection};

#[derive(Debug, Clone, PartialEq)]
pub struct TaskFilter {
    pub task_types: Vec<TaskType>,
    pub reference_id: Option<Uuid>,
    pub status: Option<TaskStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub per_page: u64,
    pub cursor: Option<KeysetCursor<TaskId>>,
    pub direction: SortDirection,
}

impl TaskFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_task_type(mut self, task_type: impl Into<TaskType>) -> Self {
        self.task_types.push(task_type.into());
        self
    }

    pub fn with_reference_id(mut self, reference_id: Uuid) -> Self {
        self.reference_id = Some(reference_id);
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn pending_only(self) -> Self {
        self.with_status(TaskStatus::Pending)
    }
}

impl Default for TaskFilter {
    fn default() -> Self {
        Self {
            task_types: vec![],
            reference_id: None,
            status: None,
            start_date: None,
            end_date: None,
            per_page: 50,
            cursor: None,
            direction: SortDirection::default(),
        }
    }
}
