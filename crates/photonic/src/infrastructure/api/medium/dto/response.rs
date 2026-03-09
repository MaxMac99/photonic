use std::collections::HashMap;

use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, Utc};
use mime_serde_shim::Wrapper as Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{FileLocationDto, MediumItemTypeDto, MediumTypeDto, StorageTierDto};
use crate::{
    domain::{
        medium::{Medium, MediumItem, MediumListItem, StorageTier},
        metadata::{CameraInfo, FileInfo, LocationInfo, Metadata, Orientation, TechnicalInfo},
    },
    infrastructure::serde::serialize_byte_as_u64,
};

/// Response for listing media - optimized for list views with minimal data
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MediumListResponse {
    pub id: Uuid,
    pub medium_type: MediumTypeDto,
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

/// Response for detailed medium view - includes all metadata
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MediumDetailResponse {
    pub id: Uuid,
    pub medium_type: MediumTypeDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_make: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_model: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub items: Vec<MediumItemDetailResponse>,
}

/// Minimal item response for list views
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MediumItemResponse {
    pub id: Uuid,
    pub is_primary: bool,
    pub medium_item_type: MediumItemTypeDto,
    #[schema(value_type = String)]
    pub mime: Mime,
    pub filename: String,
    #[schema(value_type = u64)]
    #[serde(serialize_with = "serialize_byte_as_u64")]
    pub filesize: Byte,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

/// Detailed item response for detailed views
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MediumItemDetailResponse {
    pub id: Uuid,
    pub is_primary: bool,
    pub medium_item_type: MediumItemTypeDto,
    #[schema(value_type = String)]
    pub mime: Mime,
    pub filename: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<FileLocationDto>,
    #[schema(value_type = u64)]
    #[serde(serialize_with = "serialize_byte_as_u64")]
    pub filesize: Byte,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Metadata DTOs
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MediumMetadataDto {
    pub file_info: FileInfoDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_info: Option<CameraInfoDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationInfoDto>,
    pub technical: TechnicalInfoDto,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub additional: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileInfoDto {
    #[schema(value_type = String)]
    pub mime_type: Mime,
    pub file_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_modified_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CameraInfoDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_date: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_date: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens_make: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f_number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iso: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focal_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LocationInfoDto {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_position_error: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TechnicalInfoDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<OrientationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrientationDto {
    Normal,
    MirrorHorizontal,
    Rotate180,
    MirrorVertical,
    MirrorHorizontalAndRotate270Cw,
    Rotate90Cw,
    MirrorHorizontalAndRotate90Cw,
    Rotate270Cw,
}

// Conversion implementations

impl From<(&MediumItem, bool)> for MediumItemResponse {
    fn from((item, is_primary): (&MediumItem, bool)) -> Self {
        Self {
            id: item.id,
            is_primary,
            medium_item_type: MediumItemTypeDto::from(item.medium_item_type),
            mime: Mime(item.mime.clone()),
            filename: item.filename.as_str().to_string(),
            filesize: item.filesize,
            width: item.dimensions.as_ref().map(|d| d.width() as i32),
            height: item.dimensions.as_ref().map(|d| d.height() as i32),
        }
    }
}

impl From<(&MediumItem, bool)> for MediumItemDetailResponse {
    fn from((item, is_primary): (&MediumItem, bool)) -> Self {
        Self {
            id: item.id,
            is_primary,
            medium_item_type: MediumItemTypeDto::from(item.medium_item_type),
            mime: Mime(item.mime.clone()),
            filename: item.filename.as_str().to_string(),
            locations: item
                .locations
                .iter()
                .map(|loc| FileLocationDto {
                    storage_tier: match loc.storage_tier {
                        StorageTier::Permanent => StorageTierDto::Permanent,
                        StorageTier::Temporary => StorageTierDto::Temporary,
                        StorageTier::Cache => StorageTierDto::Cache,
                    },
                    relative_path: loc.relative_path.to_string_lossy().to_string(),
                })
                .collect(),
            filesize: item.filesize,
            priority: item.priority.value(),
            width: item.dimensions.as_ref().map(|d| d.width() as i32),
            height: item.dimensions.as_ref().map(|d| d.height() as i32),
            created_at: item.created_at,
        }
    }
}

impl From<&MediumListItem> for MediumListResponse {
    fn from(list_item: &MediumListItem) -> Self {
        Self {
            id: list_item.id,
            medium_type: MediumTypeDto::from(list_item.medium_type),
            album_id: None, // TODO: Add album_id to domain entity
            taken_at: list_item.taken_at,
            camera_make: list_item.camera_make.clone(),
            camera_model: list_item.camera_model.clone(),
            items: list_item
                .items
                .iter()
                .map(|item| (item, item.id == list_item.leading_item_id).into())
                .collect(),
        }
    }
}

impl From<Medium> for MediumDetailResponse {
    fn from(medium: Medium) -> Self {
        Self {
            id: medium.id,
            medium_type: MediumTypeDto::from(medium.medium_type),
            album_id: None, // TODO: Add album_id to domain entity
            taken_at: medium.taken_at,
            camera_make: medium.camera_make.clone(),
            camera_model: medium.camera_model.clone(),
            created_at: medium.created_at.into(),
            updated_at: medium.updated_at.into(),
            items: medium
                .items
                .iter()
                .map(|item| (item, item.id == medium.leading_item_id).into())
                .collect(),
        }
    }
}

// Metadata conversion implementations

impl From<&Metadata> for MediumMetadataDto {
    fn from(metadata: &Metadata) -> Self {
        Self {
            file_info: (&metadata.file_info).into(),
            camera_info: metadata.camera_info.as_ref().map(|c| c.into()),
            location: metadata.location.as_ref().map(|l| l.into()),
            technical: (&metadata.technical).into(),
            additional: metadata.additional.clone(),
        }
    }
}

impl From<&FileInfo> for FileInfoDto {
    fn from(info: &FileInfo) -> Self {
        Self {
            mime_type: Mime(info.mime_type.0.clone()),
            file_size: info.file_size,
            file_modified_at: info.file_modified_at,
        }
    }
}

impl From<&CameraInfo> for CameraInfoDto {
    fn from(info: &CameraInfo) -> Self {
        Self {
            make: info.make.clone(),
            model: info.model.clone(),
            capture_date: info.capture_date,
            modified_date: info.modified_date,
            lens_make: info.lens_make.clone(),
            lens_model: info.lens_model.clone(),
            exposure_time: info.exposure_time,
            f_number: info.f_number,
            iso: info.iso,
            focal_length: info.focal_length,
            flash: info.flash,
        }
    }
}

impl From<&LocationInfo> for LocationInfoDto {
    fn from(info: &LocationInfo) -> Self {
        Self {
            latitude: info.latitude,
            longitude: info.longitude,
            altitude: info.altitude,
            direction: info.direction,
            horizontal_position_error: info.horizontal_position_error,
        }
    }
}

impl From<&TechnicalInfo> for TechnicalInfoDto {
    fn from(info: &TechnicalInfo) -> Self {
        Self {
            width: info.width,
            height: info.height,
            orientation: info.orientation.as_ref().map(|o| o.into()),
        }
    }
}

impl From<&Orientation> for OrientationDto {
    fn from(orientation: &Orientation) -> Self {
        match orientation {
            Orientation::Normal => OrientationDto::Normal,
            Orientation::MirrorHorizontal => OrientationDto::MirrorHorizontal,
            Orientation::Rotate180 => OrientationDto::Rotate180,
            Orientation::MirrorVertical => OrientationDto::MirrorVertical,
            Orientation::MirrorHorizontalAndRotate270CW => {
                OrientationDto::MirrorHorizontalAndRotate270Cw
            }
            Orientation::Rotate90CW => OrientationDto::Rotate90Cw,
            Orientation::MirrorHorizontalAndRotate90CW => {
                OrientationDto::MirrorHorizontalAndRotate90Cw
            }
            Orientation::Rotate270CW => OrientationDto::Rotate270Cw,
        }
    }
}
