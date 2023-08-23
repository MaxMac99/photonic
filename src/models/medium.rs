use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Utc};
use mime::Mime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediumItem {
    pub id: Option<ObjectId>,
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime: Mime,
    pub filename: String,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub filesize: u64,
    pub last_saved: DateTime<Utc>,
    pub original_store: bool,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sidecar {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Medium {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub medium_type: MediumType,
    pub date_taken: DateTime<FixedOffset>,
    pub originals: Vec<MediumItem>,
    pub album: Option<ObjectId>,
    pub tags: Vec<String>,
    pub preview: Option<MediumItem>,
    pub edits: Vec<MediumItem>,
    pub sidecars: Vec<Sidecar>,
}