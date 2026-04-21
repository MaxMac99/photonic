use async_trait::async_trait;
use domain::medium::events::{MediumCreatedEvent, MediumItemCreatedEvent, MediumUpdatedEvent};
use event_sourcing::{
    error::{EventSourcingError, Result},
    projection::handler::ProjectionHandler,
};
use sqlx::{Postgres, Transaction};
use tracing::info;

use super::{register_event, RegisterProjection};
use crate::persistence::postgres::medium::types::{MediumItemTypeDb, MediumTypeDb, StorageTierDb};

/// Projection that maintains the media, medium_items, and locations read model tables.
pub struct MediumProjection;

impl MediumProjection {
    pub fn new() -> Self {
        Self
    }
}

impl RegisterProjection for MediumProjection {
    fn register(
        bus: &super::PgProjectionBus,
        registry: &mut super::EventTypeRegistry,
    ) -> Result<()> {
        register_event::<MediumCreatedEvent, _>(bus, registry, Self::new())?;
        register_event::<MediumItemCreatedEvent, _>(bus, registry, Self::new())?;
        register_event::<MediumUpdatedEvent, _>(bus, registry, Self::new())?;
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<MediumCreatedEvent, i64, Transaction<'static, Postgres>>
    for MediumProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &MediumCreatedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        let medium_type_db = MediumTypeDb::from(event.medium_type);
        let item = &event.initial_item;
        let item_type_db = MediumItemTypeDb::from(item.medium_item_type);
        let location = item.locations.first().expect("Item must have a location");
        let storage_tier_db = StorageTierDb::from(location.storage_tier.clone());

        sqlx::query(
            "INSERT INTO media (id, owner_id, medium_type, leading_item_id, updated_at) \
             VALUES ($1, $2, $3, $4, NOW()) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(event.medium_id)
        .bind(event.user_id)
        .bind(medium_type_db as MediumTypeDb)
        .bind(item.id)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert media: {}", e),
        })?;

        sqlx::query(
            "INSERT INTO medium_items (id, medium_id, medium_item_type, mime, filename, size, priority, width, height, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW()) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(item.id)
        .bind(event.medium_id)
        .bind(item_type_db as MediumItemTypeDb)
        .bind(item.mime.to_string())
        .bind(item.filename.as_str())
        .bind(item.filesize.as_u64() as i64)
        .bind(item.priority.value())
        .bind(item.dimensions.as_ref().map(|d| d.width() as i32))
        .bind(item.dimensions.as_ref().map(|d| d.height() as i32))
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert medium_item: {}", e),
        })?;

        sqlx::query(
            "INSERT INTO locations (item_id, path, variant) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (item_id, variant) DO NOTHING",
        )
        .bind(item.id)
        .bind(location.relative_path.to_str().unwrap_or(""))
        .bind(storage_tier_db as StorageTierDb)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert location: {}", e),
        })?;

        info!(medium_id = %event.medium_id, "MediumProjection: media + initial item created");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<MediumItemCreatedEvent, i64, Transaction<'static, Postgres>>
    for MediumProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &MediumItemCreatedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        let item_type_db = MediumItemTypeDb::from(event.item_type);
        let storage_tier_db = StorageTierDb::from(event.file_location.storage_tier.clone());

        sqlx::query(
            "INSERT INTO medium_items (id, medium_id, medium_item_type, mime, filename, size, priority, width, height, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW()) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(event.item_id)
        .bind(event.medium_id)
        .bind(item_type_db as MediumItemTypeDb)
        .bind(event.mime_type.to_string())
        .bind(event.filename.as_str())
        .bind(event.filesize.as_u64() as i64)
        .bind(event.priority.value())
        .bind(event.dimensions.as_ref().map(|d| d.width() as i32))
        .bind(event.dimensions.as_ref().map(|d| d.height() as i32))
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert medium_item: {}", e),
        })?;

        sqlx::query(
            "INSERT INTO locations (item_id, path, variant) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (item_id, variant) DO NOTHING",
        )
        .bind(event.item_id)
        .bind(event.file_location.relative_path.to_str().unwrap_or(""))
        .bind(storage_tier_db as StorageTierDb)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert location: {}", e),
        })?;

        info!(
            medium_id = %event.medium_id,
            item_id = %event.item_id,
            "MediumProjection: medium item created"
        );
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<MediumUpdatedEvent, i64, Transaction<'static, Postgres>>
    for MediumProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &MediumUpdatedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        let (gps_lat, gps_lng, gps_alt) = event
            .gps_coordinates
            .map(|gps| (Some(gps.latitude()), Some(gps.longitude()), gps.altitude()))
            .unwrap_or((None, None, None));

        let (taken_at_utc, taken_at_tz) = event
            .taken_at
            .as_ref()
            .map(|dt| {
                (
                    Some(dt.with_timezone(&chrono::Utc)),
                    Some(dt.offset().local_minus_utc()),
                )
            })
            .unwrap_or((None, None));

        sqlx::query(
            "UPDATE media SET \
             taken_at = $2, taken_at_timezone = $3, \
             camera_make = $4, camera_model = $5, \
             gps_latitude = $6, gps_longitude = $7, gps_altitude = $8, \
             updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(event.medium_id)
        .bind(taken_at_utc)
        .bind(taken_at_tz)
        .bind(&event.camera_make)
        .bind(&event.camera_model)
        .bind(gps_lat)
        .bind(gps_lng)
        .bind(gps_alt)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to update media: {}", e),
        })?;

        info!(medium_id = %event.medium_id, "MediumProjection: media record updated");
        Ok(())
    }
}
