use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};

use fotonic::{
    model::{Medium, MediumItem, MediumType},
    ObjectId,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MediumItemOverview {
    #[serde(
        serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string"
    )]
    pub id: ObjectId,
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime: Mime,
    pub filename: String,
    pub width: u32,
    pub height: u32,
    pub filesize: u64,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediumOverview {
    #[serde(
        serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string"
    )]
    pub id: ObjectId,
    #[serde(rename = "mediumType")]
    pub medium_type: MediumType,
    #[serde(rename = "dateTaken")]
    pub date_taken: DateTime<Utc>,
    pub timezone: i32,
    pub originals: Vec<MediumItemOverview>,
    pub album: Option<ObjectId>,
    pub tags: Vec<String>,
    pub preview: Option<MediumItemOverview>,
    pub edits: Vec<MediumItemOverview>,
}

impl From<Medium> for MediumOverview {
    fn from(value: Medium) -> Self {
        Self {
            id: value.id.unwrap(),
            medium_type: value.medium_type,
            date_taken: value.date_taken,
            timezone: value.timezone,
            originals: value
                .originals
                .into_iter()
                .map(MediumItemOverview::from)
                .collect(),
            album: value.album,
            tags: value.tags,
            preview: value.preview.map(MediumItemOverview::from),
            edits: value
                .edits
                .into_iter()
                .map(MediumItemOverview::from)
                .collect(),
        }
    }
}

impl From<MediumItem> for MediumItemOverview {
    fn from(value: MediumItem) -> Self {
        Self {
            id: value.id,
            mime: value.mime,
            filename: value.filename,
            width: value.width,
            height: value.height,
            filesize: value.filesize,
            priority: value.priority,
        }
    }
}
