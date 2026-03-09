use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::task::TaskType;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskTypeDto {
    MetadataExtraction,
    TempCleanup,
}

impl From<TaskType> for TaskTypeDto {
    fn from(mt: TaskType) -> Self {
        match mt {
            TaskType::MetadataExtraction => TaskTypeDto::MetadataExtraction,
            TaskType::TempCleanup => TaskTypeDto::TempCleanup,
        }
    }
}

impl From<TaskTypeDto> for TaskType {
    fn from(dto: TaskTypeDto) -> Self {
        match dto {
            TaskTypeDto::MetadataExtraction => TaskType::MetadataExtraction,
            TaskTypeDto::TempCleanup => TaskType::TempCleanup,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatusDto {
    Pending,
    InProgress,
    Completed,
    Failed,
}
