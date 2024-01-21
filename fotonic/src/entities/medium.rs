use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use mime::Mime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MediumItem {
    pub id: Option<ObjectId>,
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime: Mime,
    pub filename: String,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub filesize: u64,
    #[serde(
        rename = "lastSaved",
        with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime"
    )]
    pub last_saved: DateTime<Utc>,
    #[serde(rename = "originalStore")]
    pub original_store: bool,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Sidecar {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Medium {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(rename = "mediumType")]
    pub medium_type: MediumType,
    #[serde(
        rename = "dateTaken",
        with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime"
    )]
    pub date_taken: DateTime<Utc>,
    pub timezone: i32,
    pub originals: Vec<MediumItem>,
    pub album: Option<ObjectId>,
    pub tags: Vec<String>,
    pub preview: Option<MediumItem>,
    pub edits: Vec<MediumItem>,
    pub sidecars: Vec<Sidecar>,
    pub additional_data: HashMap<String, String>,
}
