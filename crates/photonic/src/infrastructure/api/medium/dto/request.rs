use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_default_utils::*;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use super::types::MediumTypeDto;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub enum DirectionDto {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct CreateMediumInput {
    #[serde(default)]
    pub tags: Vec<String>,
    pub medium_type: Option<MediumTypeDto>,
    pub album_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct CreateMediumItemInput {
    pub filename: String,
    #[serde(default = "default_i32::<10>")]
    #[param(default = 10)]
    pub priority: i32,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct FindAllMediaOptions {
    pub start_date: Option<DateTime<chrono::Utc>>,
    pub end_date: Option<DateTime<chrono::Utc>>,
    #[serde(default = "default_u64::<50>")]
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: u64,
    pub page_last_date: Option<DateTime<chrono::Utc>>,
    pub page_last_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub album_id: Option<Uuid>,
    #[serde(default)]
    #[param(inline, default = "Desc")]
    pub direction: DirectionDto,
    #[serde(default)]
    #[param(default = false)]
    pub include_no_album: bool,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct GetMediumPreviewOptions {
    pub width: Option<i32>,
    pub height: Option<i32>,
}
