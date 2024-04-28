use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use diesel_derive_enum::DbEnum;
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::MediumTypeEnum"]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::StoreLocationEnum"]
pub enum StoreLocation {
    Originals,
    Cache,
}

#[derive(Debug, Serialize, Deserialize, DbEnum, PartialEq, Eq, Hash, Copy, Clone)]
#[ExistingTypePath = "crate::schema::sql_types::MediumItemTypeEnum"]
pub enum MediumItemType {
    Original,
    Edit,
    Preview,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub id: Uuid,
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime: Mime,
    pub filename: String,
    pub path: PathBuf,
    pub filesize: u64,
    pub priority: i32,
    #[serde(rename = "lastSaved")]
    pub last_saved: NaiveDateTime,
    pub location: StoreLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediumItem {
    pub file: FileItem,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "dateTaken")]
    pub date_taken: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Medium {
    pub id: Uuid,
    pub owner: Uuid,
    #[serde(rename = "mediumType")]
    pub medium_type: MediumType,
    pub originals: Vec<MediumItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<Uuid>,
    pub tags: Vec<String>,
    pub previews: Vec<MediumItem>,
    pub edits: Vec<MediumItem>,
    pub sidecars: Vec<FileItem>,
}
