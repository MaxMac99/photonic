use async_trait::async_trait;
use domain::metadata::events::{
    MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
};
use event_sourcing::{
    error::{EventSourcingError, Result},
    projection::handler::ProjectionHandler,
};
use sqlx::{Postgres, Transaction};
use tracing::info;

use super::{register_event, RegisterProjection};
use crate::persistence::postgres::metadata::entity::OrientationDb;

/// Projection that maintains the metadata read model table.
pub struct MetadataProjection;

impl MetadataProjection {
    pub fn new() -> Self {
        Self
    }
}

impl RegisterProjection for MetadataProjection {
    fn register(
        bus: &super::PgProjectionBus,
        registry: &mut super::EventTypeRegistry,
    ) -> Result<()> {
        register_event::<MetadataExtractionStartedEvent, _>(bus, registry, Self::new())?;
        register_event::<MetadataExtractedEvent, _>(bus, registry, Self::new())?;
        register_event::<MetadataExtractionFailedEvent, _>(bus, registry, Self::new())?;
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<MetadataExtractionStartedEvent, i64, Transaction<'static, Postgres>>
    for MetadataProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        _event: &MetadataExtractionStartedEvent,
        _sequence: i64,
        _tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        // No read model update needed when extraction starts
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<MetadataExtractedEvent, i64, Transaction<'static, Postgres>>
    for MetadataProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &MetadataExtractedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        let m = &event.metadata;
        let camera = m.camera_info.as_ref();
        let loc = m.location.as_ref();
        let orientation_db = m.technical.orientation.as_ref().map(OrientationDb::from);

        sqlx::query(
            "INSERT INTO metadata (
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
                additional = EXCLUDED.additional",
        )
        .bind(m.id)
        .bind(m.medium_id)
        .bind(m.extracted_at)
        // File info
        .bind(m.file_info.mime_type.to_string())
        .bind(m.file_info.file_size as i64)
        .bind(
            m.file_info
                .file_modified_at
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        )
        // Camera info
        .bind(camera.and_then(|c| c.make.as_deref()))
        .bind(camera.and_then(|c| c.model.as_deref()))
        .bind(camera.and_then(|c| c.capture_date.map(|dt| dt.with_timezone(&chrono::Utc))))
        .bind(camera.and_then(|c| c.modified_date.map(|dt| dt.with_timezone(&chrono::Utc))))
        .bind(camera.and_then(|c| c.lens_make.as_deref()))
        .bind(camera.and_then(|c| c.lens_model.as_deref()))
        .bind(camera.and_then(|c| c.exposure_time))
        .bind(camera.and_then(|c| c.f_number))
        .bind(camera.and_then(|c| c.iso.map(|i| i as i16)))
        .bind(camera.and_then(|c| c.focal_length))
        .bind(camera.and_then(|c| c.flash))
        // Location
        .bind(loc.map(|l| l.latitude))
        .bind(loc.map(|l| l.longitude))
        .bind(loc.and_then(|l| l.altitude))
        .bind(loc.and_then(|l| l.direction))
        .bind(loc.and_then(|l| l.horizontal_position_error))
        // Technical
        .bind(m.technical.width.map(|w| w as i32))
        .bind(m.technical.height.map(|h| h as i32))
        .bind(orientation_db)
        // Additional
        .bind(sqlx::types::Json(&m.additional))
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to upsert metadata: {}", e),
        })?;

        info!(medium_id = %event.medium_id, "MetadataProjection: metadata upserted");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<MetadataExtractionFailedEvent, i64, Transaction<'static, Postgres>>
    for MetadataProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        _event: &MetadataExtractionFailedEvent,
        _sequence: i64,
        _tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        // No read model update needed when extraction fails
        Ok(())
    }
}
