use crate::common::Direction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateAlbumInput {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct FindAllAlbumsOptions {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default)]
    #[param(default = false)]
    pub include_empty_albums: bool,
    #[serde(default = "default_page_size")]
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: u64,
    pub page_last_date: Option<DateTime<Utc>>,
    pub page_last_id: Option<Uuid>,
    #[serde(default)]
    #[param(inline, default = "Desc")]
    pub direction: Direction,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AlbumResponse {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub number_of_items: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_date: Option<DateTime<Utc>>,
}

fn default_page_size() -> u64 {
    50
}
