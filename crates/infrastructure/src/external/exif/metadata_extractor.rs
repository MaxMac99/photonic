use std::{collections::HashMap, str::FromStr, sync::Arc};

use application::{medium::ports::FileStorage, metadata::ports::MetadataExtractor};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use domain::{
    error::DomainResult,
    medium::{FileLocation, MediumId},
    metadata::{CameraInfo, FileInfo, LocationInfo, Metadata, Orientation, TechnicalInfo},
};
use uuid::Uuid;

use super::{Exiftool, Field};

/// Infrastructure adapter that implements MetadataExtractor using exiftool
pub struct ExiftoolMetadataExtractor {
    exiftool: Arc<Exiftool>,
    file_storage: Arc<dyn FileStorage>,
}

impl ExiftoolMetadataExtractor {
    pub fn new(exiftool: Arc<Exiftool>, file_storage: Arc<dyn FileStorage>) -> Self {
        Self {
            exiftool,
            file_storage,
        }
    }
}

#[async_trait]
impl MetadataExtractor for ExiftoolMetadataExtractor {
    async fn extract(
        &self,
        location: &FileLocation,
        medium_id: MediumId,
    ) -> DomainResult<Metadata> {
        let path = self.file_storage.get_local_path(location).await?;
        let exif_data = self.exiftool.read_file(path, false).await?;

        // Convert exiftool output to domain Metadata
        Ok(convert_exif_to_metadata(&exif_data, medium_id))
    }
}

/// Convert exiftool's raw output to our domain Metadata
fn convert_exif_to_metadata(exif: &HashMap<String, Field>, medium_id: MediumId) -> Metadata {
    // Extract basic file info
    let file_info = FileInfo {
        mime_type: exif
            .get("MIMEType")
            .and_then(|v| v.value.as_str())
            .and_then(|v| mime::Mime::from_str(v).ok())
            .unwrap_or(mime::APPLICATION_OCTET_STREAM)
            .into(),
        file_size: exif
            .get("FileSize")
            .and_then(|v| v.raw.as_ref())
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        file_modified_at: exif
            .get("FileModifyDate")
            .and_then(|v| v.value.as_str())
            .and_then(parse_exif_date),
    };

    // Extract camera info
    let camera_info = if has_camera_info(exif) {
        Some(CameraInfo {
            make: exif
                .get("Make")
                .and_then(|v| v.value.as_str())
                .map(String::from),
            model: exif
                .get("Model")
                .and_then(|v| v.value.as_str())
                .map(String::from),
            capture_date: exif
                .get("SubSecDateTimeOriginal")
                .and_then(|v| v.value.as_str())
                .and_then(parse_exif_date)
                .or(exif
                    .get("DateTimeOriginal")
                    .and_then(|v| v.value.as_str())
                    .and_then(parse_exif_date)),
            modified_date: exif
                .get("SubSecModifyDate")
                .and_then(|v| v.value.as_str())
                .and_then(parse_exif_date),
            lens_make: exif
                .get("LensMake")
                .and_then(|v| v.value.as_str())
                .map(String::from),
            lens_model: exif
                .get("LensModel")
                .and_then(|v| v.value.as_str())
                .map(String::from),
            exposure_time: exif.get("ExposureTime").and_then(|v| v.value.as_f64()),
            f_number: exif
                .get("FNumber")
                .and_then(|v| v.raw.as_ref())
                .and_then(|v| v.as_f64()),
            iso: exif
                .get("ISO")
                .and_then(|v| v.value.as_u64())
                .map(|i| i as u16),
            focal_length: exif
                .get("FocalLength")
                .and_then(|v| v.raw.as_ref())
                .and_then(|v| v.as_f64()),
            flash: exif
                .get("Flash")
                .and_then(|v| v.value.as_str())
                .map(|s| s.contains("Fired")),
        })
    } else {
        None
    };

    // Extract location info
    let location = extract_location(exif);

    // Extract technical info
    let technical = TechnicalInfo {
        width: exif
            .get("ImageWidth")
            .and_then(|v| v.value.as_u64())
            .map(|w| w as u32),
        height: exif
            .get("ImageHeight")
            .and_then(|v| v.value.as_u64())
            .map(|h| h as u32),
        orientation: exif
            .get("Orientation")
            .and_then(|v| v.raw.as_ref())
            .and_then(|v| v.as_u64())
            .map(|o| Orientation::from(o as u8)),
    };

    // Collect any additional EXIF fields not captured in structured fields
    let mut additional = HashMap::new();
    for (key, field) in exif.iter() {
        // Skip fields we've already extracted
        let is_extracted = matches!(
            key.as_str(),
            "MIMEType"
                | "FileSize"
                | "FileModifyDate"
                | "Make"
                | "Model"
                | "SubSecDateTimeOriginal"
                | "DateTimeOriginal"
                | "SubSecModifyDate"
                | "LensMake"
                | "LensModel"
                | "ExposureTime"
                | "FNumber"
                | "ISO"
                | "FocalLength"
                | "Flash"
                | "GPSLatitude"
                | "GPSLongitude"
                | "GPSAltitude"
                | "GPSImgDirection"
                | "GPSHPositioningError"
                | "ImageWidth"
                | "ImageHeight"
                | "Orientation"
        );

        if !is_extracted {
            if let Some(value_str) = field.value.as_str() {
                additional.insert(key.clone(), value_str.to_string());
            }
        }
    }

    Metadata {
        id: Uuid::new_v4(),
        medium_id,
        extracted_at: Utc::now(),
        file_info,
        camera_info,
        location,
        technical,
        additional,
    }
}

fn has_camera_info(exif: &HashMap<String, Field>) -> bool {
    exif.get("Make").is_some()
        || exif.get("Model").is_some()
        || exif.get("SubSecDateTimeOriginal").is_some()
}

fn extract_location(exif: &HashMap<String, Field>) -> Option<LocationInfo> {
    let lat = exif
        .get("GPSLatitude")
        .and_then(|v| v.raw.as_ref().and_then(|v| v.as_f64()));
    let lon = exif
        .get("GPSLongitude")
        .and_then(|v| v.raw.as_ref().and_then(|v| v.as_f64()));

    if let (Some(latitude), Some(longitude)) = (lat, lon) {
        Some(LocationInfo {
            latitude,
            longitude,
            altitude: exif
                .get("GPSAltitude")
                .and_then(|v| v.raw.as_ref().and_then(|v| v.as_f64())),
            direction: exif
                .get("GPSImgDirection")
                .and_then(|v| v.raw.as_ref().and_then(|v| v.as_f64())),
            horizontal_position_error: exif
                .get("GPSHPositioningError")
                .and_then(|v| v.raw.as_ref().and_then(|v| v.as_f64())),
        })
    } else {
        None
    }
}

fn parse_exif_date(date_str: &str) -> Option<DateTime<FixedOffset>> {
    // Parse various EXIF date formats
    // Common format: "2024:01:15 14:30:45"
    DateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S%.f%:z")
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S")
                .ok()
                .and_then(|val| {
                    FixedOffset::east_opt(0)
                        .unwrap()
                        .from_local_datetime(&val)
                        .earliest()
                })
        })
}
