use domain::{error::DomainResult, metadata::Metadata};
use sqlx::types::Json;

use super::{entity::OrientationDb, PostgresMetadataRepository};
use crate::persistence::postgres::repo_error;

impl PostgresMetadataRepository {
    pub(super) async fn save_impl(&self, metadata: &Metadata) -> DomainResult<()> {
        let orientation: Option<OrientationDb> =
            metadata.technical.orientation.as_ref().map(Into::into);

        sqlx::query(
            r#"
            INSERT INTO metadata (
                id, medium_id, extracted_at,
                mime_type, file_size, file_modified_at,
                camera_make, camera_model, capture_date, modified_date,
                lens_make, lens_model, exposure_time, f_number, iso, focal_length, flash,
                latitude, longitude, altitude, direction, horizontal_position_error,
                width, height, orientation,
                additional
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22,
                $23, $24, $25,
                $26
            )
            ON CONFLICT (medium_id) DO UPDATE SET
                extracted_at = EXCLUDED.extracted_at,
                mime_type = EXCLUDED.mime_type,
                file_size = EXCLUDED.file_size,
                file_modified_at = EXCLUDED.file_modified_at,
                camera_make = EXCLUDED.camera_make,
                camera_model = EXCLUDED.camera_model,
                capture_date = EXCLUDED.capture_date,
                modified_date = EXCLUDED.modified_date,
                lens_make = EXCLUDED.lens_make,
                lens_model = EXCLUDED.lens_model,
                exposure_time = EXCLUDED.exposure_time,
                f_number = EXCLUDED.f_number,
                iso = EXCLUDED.iso,
                focal_length = EXCLUDED.focal_length,
                flash = EXCLUDED.flash,
                latitude = EXCLUDED.latitude,
                longitude = EXCLUDED.longitude,
                altitude = EXCLUDED.altitude,
                direction = EXCLUDED.direction,
                horizontal_position_error = EXCLUDED.horizontal_position_error,
                width = EXCLUDED.width,
                height = EXCLUDED.height,
                orientation = EXCLUDED.orientation,
                additional = EXCLUDED.additional
            "#,
        )
        .bind(metadata.id)
        .bind(metadata.medium_id)
        .bind(metadata.extracted_at)
        // File info
        .bind(metadata.file_info.mime_type.to_string())
        .bind(metadata.file_info.file_size as i64)
        .bind(metadata.file_info.file_modified_at.map(|dt| dt.to_utc()))
        // Camera info
        .bind(metadata.camera_info.as_ref().and_then(|c| c.make.clone()))
        .bind(metadata.camera_info.as_ref().and_then(|c| c.model.clone()))
        .bind(
            metadata
                .camera_info
                .as_ref()
                .and_then(|c| c.capture_date.map(|dt| dt.to_utc())),
        )
        .bind(
            metadata
                .camera_info
                .as_ref()
                .and_then(|c| c.modified_date.map(|dt| dt.to_utc())),
        )
        .bind(
            metadata
                .camera_info
                .as_ref()
                .and_then(|c| c.lens_make.clone()),
        )
        .bind(
            metadata
                .camera_info
                .as_ref()
                .and_then(|c| c.lens_model.clone()),
        )
        .bind(metadata.camera_info.as_ref().and_then(|c| c.exposure_time))
        .bind(metadata.camera_info.as_ref().and_then(|c| c.f_number))
        .bind(
            metadata
                .camera_info
                .as_ref()
                .and_then(|c| c.iso.map(|i| i as i16)),
        )
        .bind(metadata.camera_info.as_ref().and_then(|c| c.focal_length))
        .bind(metadata.camera_info.as_ref().and_then(|c| c.flash))
        // Location
        .bind(metadata.location.as_ref().map(|l| l.latitude))
        .bind(metadata.location.as_ref().map(|l| l.longitude))
        .bind(metadata.location.as_ref().and_then(|l| l.altitude))
        .bind(metadata.location.as_ref().and_then(|l| l.direction))
        .bind(
            metadata
                .location
                .as_ref()
                .and_then(|l| l.horizontal_position_error),
        )
        // Technical
        .bind(metadata.technical.width.map(|w| w as i32))
        .bind(metadata.technical.height.map(|h| h as i32))
        .bind(orientation)
        // Additional
        .bind(Json(&metadata.additional))
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(())
    }
}
