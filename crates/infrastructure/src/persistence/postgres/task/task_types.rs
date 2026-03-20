use domain::task::{status::TaskStatus, TaskType};

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "task_type_enum", rename_all = "snake_case")]
pub enum TaskTypeDb {
    MetadataExtraction,
    TempCleanup,
}

impl From<TaskTypeDb> for TaskType {
    fn from(task_type: TaskTypeDb) -> Self {
        match task_type {
            TaskTypeDb::MetadataExtraction => TaskType::MetadataExtraction,
            TaskTypeDb::TempCleanup => TaskType::TempCleanup,
        }
    }
}

impl From<TaskType> for TaskTypeDb {
    fn from(task_type: TaskType) -> Self {
        match task_type {
            TaskType::MetadataExtraction => TaskTypeDb::MetadataExtraction,
            TaskType::TempCleanup => TaskTypeDb::TempCleanup,
        }
    }
}

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "task_status_enum", rename_all = "snake_case")]
pub enum TaskStatusDb {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl From<TaskStatus> for TaskStatusDb {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => TaskStatusDb::Pending,
            TaskStatus::InProgress => TaskStatusDb::InProgress,
            TaskStatus::Completed => TaskStatusDb::Completed,
            TaskStatus::Failed(_) => TaskStatusDb::Failed,
        }
    }
}
