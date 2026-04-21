use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use uuid::Uuid;

use crate::api::task::dto::{TaskStatusDto, TaskTypeDto};

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
