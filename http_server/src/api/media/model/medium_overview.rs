use chrono::{DateTime, FixedOffset};
use mime::Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use fotonic::model::{FileItem, Medium, MediumItem, MediumType};

#[derive(Debug, Serialize, Deserialize)]
pub struct MediumItemOverview {
    pub id: Uuid,
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime: Mime,
    pub filename: String,
    #[serde(rename = "dateTaken")]
    pub date_taken: DateTime<FixedOffset>,
    pub width: u32,
    pub height: u32,
    pub filesize: u64,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SidecarOverview {
    pub id: Uuid,
    #[serde(rename = "type", with = "mime_serde_shim")]
    pub mime: Mime,
    pub filename: String,
    pub filesize: u64,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediumOverview {
    pub id: Uuid,
    #[serde(rename = "mediumType")]
    pub medium_type: MediumType,
    pub originals: Vec<MediumItemOverview>,
    pub album: Option<Uuid>,
    pub tags: Vec<String>,
    pub preview: Option<MediumItemOverview>,
    pub edits: Vec<MediumItemOverview>,
    pub sidecars: Vec<SidecarOverview>,
}

impl From<Medium> for MediumOverview {
    fn from(value: Medium) -> Self {
        Self {
            id: value.id,
            medium_type: value.medium_type,
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
            sidecars: value
                .sidecars
                .into_iter()
                .map(SidecarOverview::from)
                .collect(),
        }
    }
}

impl From<MediumItem> for MediumItemOverview {
    fn from(value: MediumItem) -> Self {
        Self {
            id: value.file.id,
            mime: value.file.mime,
            filename: value.file.filename,
            date_taken: value.date_taken,
            width: value.width,
            height: value.height,
            filesize: value.file.filesize,
            priority: value.file.priority,
        }
    }
}

impl From<FileItem> for SidecarOverview {
    fn from(value: FileItem) -> Self {
        Self {
            id: value.id,
            mime: value.mime,
            filename: value.filename,
            filesize: value.filesize,
            priority: value.priority,
        }
    }
}
