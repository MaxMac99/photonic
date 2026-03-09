use crate::infrastructure::api::task::dto::{TaskStatusDto, TaskTypeDto};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_default_utils::*;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub enum DirectionDto {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct FindTasksOptions {
    #[serde(default)]
    pub types: Vec<TaskTypeDto>,
    #[serde(default)]
    pub states: Vec<TaskStatusDto>,
    pub start_date: Option<DateTime<chrono::Utc>>,
    pub end_date: Option<DateTime<chrono::Utc>>,
    #[serde(default = "default_u64::<50>")]
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: u64,
    pub page_last_date: Option<DateTime<chrono::Utc>>,
    pub page_last_id: Option<Uuid>,
    #[serde(default)]
    #[param(inline, default = "Desc")]
    pub direction: DirectionDto,
}
