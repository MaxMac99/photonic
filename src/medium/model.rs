use crate::{common::Direction, storage::StorageLocation, util::serde::serialize_byte_as_u64};
use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use mime_serde_shim::Wrapper as Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema, PartialEq)]
#[sqlx(type_name = "medium_type_enum", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "medium_item_type_enum", rename_all = "lowercase")]
pub enum MediumItemType {
    Original,
    Edit,
    Preview,
    Sidecar,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct CreateMediumInput {
    #[serde(default)]
    pub tags: Vec<String>,
    pub medium_type: Option<MediumType>,
    pub album_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct CreateMediumItemInput {
    pub filename: String,
    pub extension: String,
    #[serde(default = "default_prio")]
    #[param(default = 10)]
    pub priority: i32,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct FindAllMediaOptions {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default = "default_page_size")]
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: u64,
    pub page_last_date: Option<DateTime<Utc>>,
    pub page_last_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub album_id: Option<Uuid>,
    #[serde(default)]
    #[param(inline, default = "Desc")]
    pub direction: Direction,
    #[serde(default)]
    #[param(default = false)]
    pub include_no_album: bool,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct GetMediumPreviewOptions {
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MediumResponse {
    pub id: Uuid,
    pub medium_type: MediumType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_make: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_model: Option<String>,
    pub items: Vec<MediumItemResponse>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MediumItemResponse {
    pub id: Uuid,
    pub is_primary: bool,
    pub medium_item_type: MediumItemType,
    #[schema(value_type = String)]
    pub mime: Mime,
    pub filename: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<StorageLocation>,
    #[schema(value_type = u64)]
    #[serde(serialize_with = "serialize_byte_as_u64")]
    pub filesize: Byte,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    pub last_saved: NaiveDateTime,
}

impl From<mime::Mime> for MediumType {
    fn from(value: mime::Mime) -> Self {
        match (value.type_(), value.subtype()) {
            (mime::IMAGE, mime::SVG) => MediumType::Vector,
            (mime::IMAGE, mime::GIF) => MediumType::Gif,
            (mime::IMAGE, _) => MediumType::Photo,
            (mime::VIDEO, _) => MediumType::Video,
            _ => MediumType::Other,
        }
    }
}

fn default_prio() -> i32 {
    10
}

fn default_page_size() -> u64 {
    50
}
