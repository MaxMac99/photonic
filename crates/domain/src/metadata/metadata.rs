use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Utc};
use mime::Mime;
use uuid::Uuid;

use crate::{
    medium::{MediumId, MediumItemId},
    metadata::events::{
        MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
    },
    user::UserId,
};

pub type MetadataId = Uuid;

/// Metadata entity - owns full EXIF data for a medium
#[derive(Debug, Clone)]
pub struct Metadata {
    pub id: MetadataId,
    pub medium_id: MediumId,
    pub extracted_at: DateTime<Utc>,
    pub file_info: FileInfo,
    pub camera_info: Option<CameraInfo>,
    pub location: Option<LocationInfo>,
    pub technical: TechnicalInfo,
    pub additional: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub mime_type: Mime,
    pub file_size: u64,
    pub file_modified_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub capture_date: Option<DateTime<FixedOffset>>,
    pub modified_date: Option<DateTime<FixedOffset>>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<f64>,
    pub f_number: Option<f64>,
    pub iso: Option<u16>,
    pub focal_length: Option<f64>,
    pub flash: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub direction: Option<f64>,
    pub horizontal_position_error: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TechnicalInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<Orientation>,
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Normal,
    MirrorHorizontal,
    Rotate180,
    MirrorVertical,
    MirrorHorizontalAndRotate270CW,
    Rotate90CW,
    MirrorHorizontalAndRotate90CW,
    Rotate270CW,
}

impl Metadata {
    pub fn extraction_started(
        medium_id: MediumId,
        leading_item_id: MediumItemId,
        owner_id: UserId,
    ) -> MetadataExtractionStartedEvent {
        MetadataExtractionStartedEvent::new(medium_id, leading_item_id, owner_id)
    }

    pub fn extracted(
        &self,
        leading_item_id: MediumItemId,
        owner_id: UserId,
    ) -> MetadataExtractedEvent {
        MetadataExtractedEvent::new(self.medium_id, leading_item_id, owner_id, self.clone())
    }

    pub fn extraction_failed(
        medium_id: MediumId,
        leading_item_id: MediumItemId,
        owner_id: UserId,
        error: String,
    ) -> MetadataExtractionFailedEvent {
        MetadataExtractionFailedEvent::new(medium_id, leading_item_id, owner_id, error)
    }

    pub fn is_video(&self) -> bool {
        self.file_info.mime_type.type_().eq(&mime::VIDEO)
    }

    pub fn is_image(&self) -> bool {
        self.file_info.mime_type.type_().eq(&mime::IMAGE)
    }

    pub fn capture_date(&self) -> Option<DateTime<FixedOffset>> {
        self.camera_info.as_ref().and_then(|c| c.capture_date)
    }

    pub fn gps_coordinates(&self) -> Option<(f64, f64, Option<f64>)> {
        self.location
            .as_ref()
            .map(|loc| (loc.latitude, loc.longitude, loc.altitude))
    }

    pub fn has_gps(&self) -> bool {
        self.location.is_some()
    }
}

impl From<u8> for Orientation {
    fn from(value: u8) -> Self {
        match value {
            1 => Orientation::Normal,
            2 => Orientation::MirrorHorizontal,
            3 => Orientation::Rotate180,
            4 => Orientation::MirrorVertical,
            5 => Orientation::MirrorHorizontalAndRotate270CW,
            6 => Orientation::Rotate90CW,
            7 => Orientation::MirrorHorizontalAndRotate90CW,
            8 => Orientation::Rotate270CW,
            _ => Orientation::Normal,
        }
    }
}
