use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use mime_serde_shim::Wrapper as Mime;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::domain::metadata::{
    CameraInfo, FileInfo, LocationInfo, Metadata, Orientation, TechnicalInfo,
};

#[derive(Debug, sqlx::FromRow)]
pub struct MetadataDb {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub extracted_at: DateTime<Utc>,
    // File info
    pub mime_type: String,
    pub file_size: i64,
    pub file_modified_at: Option<DateTime<Utc>>,
    // Camera info
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub capture_date: Option<DateTime<Utc>>,
    pub modified_date: Option<DateTime<Utc>>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<f64>,
    pub f_number: Option<f64>,
    pub iso: Option<i16>,
    pub focal_length: Option<f64>,
    pub flash: Option<bool>,
    // Location
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub direction: Option<f64>,
    pub horizontal_position_error: Option<f64>,
    // Technical
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub orientation: Option<OrientationDb>,
    // Additional
    pub additional: Json<HashMap<String, String>>,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "orientation_enum", rename_all = "snake_case")]
pub enum OrientationDb {
    Normal,
    MirrorHorizontal,
    Rotate180,
    MirrorVertical,
    MirrorHorizontalRotate270Cw,
    Rotate90Cw,
    MirrorHorizontalRotate90Cw,
    Rotate270Cw,
}

impl From<&Orientation> for OrientationDb {
    fn from(o: &Orientation) -> Self {
        match o {
            Orientation::Normal => OrientationDb::Normal,
            Orientation::MirrorHorizontal => OrientationDb::MirrorHorizontal,
            Orientation::Rotate180 => OrientationDb::Rotate180,
            Orientation::MirrorVertical => OrientationDb::MirrorVertical,
            Orientation::MirrorHorizontalAndRotate270CW => {
                OrientationDb::MirrorHorizontalRotate270Cw
            }
            Orientation::Rotate90CW => OrientationDb::Rotate90Cw,
            Orientation::MirrorHorizontalAndRotate90CW => OrientationDb::MirrorHorizontalRotate90Cw,
            Orientation::Rotate270CW => OrientationDb::Rotate270Cw,
        }
    }
}

impl From<OrientationDb> for Orientation {
    fn from(o: OrientationDb) -> Self {
        match o {
            OrientationDb::Normal => Orientation::Normal,
            OrientationDb::MirrorHorizontal => Orientation::MirrorHorizontal,
            OrientationDb::Rotate180 => Orientation::Rotate180,
            OrientationDb::MirrorVertical => Orientation::MirrorVertical,
            OrientationDb::MirrorHorizontalRotate270Cw => {
                Orientation::MirrorHorizontalAndRotate270CW
            }
            OrientationDb::Rotate90Cw => Orientation::Rotate90CW,
            OrientationDb::MirrorHorizontalRotate90Cw => Orientation::MirrorHorizontalAndRotate90CW,
            OrientationDb::Rotate270Cw => Orientation::Rotate270CW,
        }
    }
}

impl From<MetadataDb> for Metadata {
    fn from(db: MetadataDb) -> Self {
        let file_info = FileInfo {
            mime_type: Mime(
                mime::Mime::from_str(&db.mime_type).unwrap_or(mime::APPLICATION_OCTET_STREAM),
            ),
            file_size: db.file_size as u64,
            file_modified_at: db.file_modified_at.map(|dt| dt.fixed_offset()),
        };

        let camera_info =
            if db.camera_make.is_some() || db.camera_model.is_some() || db.capture_date.is_some() {
                Some(CameraInfo {
                    make: db.camera_make,
                    model: db.camera_model,
                    capture_date: db.capture_date.map(|dt| dt.fixed_offset()),
                    modified_date: db.modified_date.map(|dt| dt.fixed_offset()),
                    lens_make: db.lens_make,
                    lens_model: db.lens_model,
                    exposure_time: db.exposure_time,
                    f_number: db.f_number,
                    iso: db.iso.map(|i| i as u16),
                    focal_length: db.focal_length,
                    flash: db.flash,
                })
            } else {
                None
            };

        let location = db
            .latitude
            .zip(db.longitude)
            .map(|(lat, lon)| LocationInfo {
                latitude: lat,
                longitude: lon,
                altitude: db.altitude,
                direction: db.direction,
                horizontal_position_error: db.horizontal_position_error,
            });

        let technical = TechnicalInfo {
            width: db.width.map(|w| w as u32),
            height: db.height.map(|h| h as u32),
            orientation: db.orientation.map(Into::into),
        };

        Metadata {
            id: db.id,
            medium_id: db.medium_id,
            extracted_at: db.extracted_at,
            file_info,
            camera_info,
            location,
            technical,
            additional: db.additional.0,
        }
    }
}
