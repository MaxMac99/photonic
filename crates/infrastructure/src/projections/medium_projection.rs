use application::error::{ApplicationError, ApplicationResult};
use application::event_bus::EventProcessor;
use async_trait::async_trait;
use domain::medium::events::MediumEvent;
use sqlx::PgPool;
use tracing::{debug, info};

use crate::persistence::postgres::medium::types::{MediumItemTypeDb, MediumTypeDb, StorageTierDb};

/// Projection that maintains the media, medium_items, and locations read model tables.
pub struct MediumProjection {
    pool: PgPool,
}

impl MediumProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventProcessor<MediumEvent> for MediumProjection {
    async fn process(&self, event: &MediumEvent) -> ApplicationResult<()> {
        debug!("MediumProjection handling event");

        match event {
            MediumEvent::MediumCreated(e) => {
                let medium_type_db = MediumTypeDb::from(e.medium_type);
                let item = &e.initial_item;
                let item_type_db = MediumItemTypeDb::from(item.medium_item_type);
                let location = item.locations.first().expect("Item must have a location");
                let storage_tier_db = StorageTierDb::from(location.storage_tier.clone());

                let mut tx = self
                    .pool
                    .begin()
                    .await
                    .map_err(|e| ApplicationError::Internal {
                        message: format!("Failed to begin transaction: {}", e),
                    })?;

                // Insert media + initial item + location in one deferred transaction
                sqlx::query(
                    "INSERT INTO media (id, owner_id, medium_type, leading_item_id, updated_at) \
                     VALUES ($1, $2, $3, $4, NOW()) \
                     ON CONFLICT (id) DO NOTHING",
                )
                .bind(e.medium_id)
                .bind(e.user_id)
                .bind(medium_type_db as MediumTypeDb)
                .bind(item.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to insert media: {}", e),
                })?;

                sqlx::query(
                    "INSERT INTO medium_items (id, medium_id, medium_item_type, mime, filename, size, priority, width, height, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW()) \
                     ON CONFLICT (id) DO NOTHING",
                )
                .bind(item.id)
                .bind(e.medium_id)
                .bind(item_type_db as MediumItemTypeDb)
                .bind(item.mime.to_string())
                .bind(item.filename.as_str())
                .bind(item.filesize.as_u64() as i64)
                .bind(item.priority.value())
                .bind(item.dimensions.as_ref().map(|d| d.width() as i32))
                .bind(item.dimensions.as_ref().map(|d| d.height() as i32))
                .execute(&mut *tx)
                .await
                .map_err(|e| ApplicationError::Internal {
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
                .execute(&mut *tx)
                .await
                .map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to insert location: {}", e),
                })?;

                tx.commit().await.map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to commit transaction: {}", e),
                })?;

                info!(medium_id = %e.medium_id, "MediumProjection: media + initial item created");
            }
            MediumEvent::MediumItemCreated(e) => {
                let item_type_db = MediumItemTypeDb::from(e.item_type);
                let storage_tier_db = StorageTierDb::from(e.file_location.storage_tier.clone());

                let mut tx = self
                    .pool
                    .begin()
                    .await
                    .map_err(|e| ApplicationError::Internal {
                        message: format!("Failed to begin transaction: {}", e),
                    })?;

                // Insert medium_item
                sqlx::query(
                    "INSERT INTO medium_items (id, medium_id, medium_item_type, mime, filename, size, priority, width, height, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW()) \
                     ON CONFLICT (id) DO NOTHING",
                )
                .bind(e.item_id)
                .bind(e.medium_id)
                .bind(item_type_db as MediumItemTypeDb)
                .bind(e.mime_type.to_string())
                .bind(e.filename.as_str())
                .bind(e.filesize.as_u64() as i64)
                .bind(e.priority.value())
                .bind(e.dimensions.as_ref().map(|d| d.width() as i32))
                .bind(e.dimensions.as_ref().map(|d| d.height() as i32))
                .execute(&mut *tx)
                .await
                .map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to insert medium_item: {}", e),
                })?;

                // Insert location
                sqlx::query(
                    "INSERT INTO locations (item_id, path, variant) \
                     VALUES ($1, $2, $3) \
                     ON CONFLICT (item_id, variant) DO NOTHING",
                )
                .bind(e.item_id)
                .bind(e.file_location.relative_path.to_str().unwrap_or(""))
                .bind(storage_tier_db as StorageTierDb)
                .execute(&mut *tx)
                .await
                .map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to insert location: {}", e),
                })?;

                tx.commit().await.map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to commit transaction: {}", e),
                })?;

                info!(
                    medium_id = %e.medium_id,
                    item_id = %e.item_id,
                    "MediumProjection: medium item created"
                );
            }
            MediumEvent::MediumUpdated(e) => {
                let (gps_lat, gps_lng, gps_alt) = e
                    .gps_coordinates
                    .map(|gps| (Some(gps.latitude()), Some(gps.longitude()), gps.altitude()))
                    .unwrap_or((None, None, None));

                let (taken_at_utc, taken_at_tz) = e
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
                .bind(e.medium_id)
                .bind(taken_at_utc)
                .bind(taken_at_tz)
                .bind(&e.camera_make)
                .bind(&e.camera_model)
                .bind(gps_lat)
                .bind(gps_lng)
                .bind(gps_alt)
                .execute(&self.pool)
                .await
                .map_err(|e| ApplicationError::Internal {
                    message: format!("Failed to update media: {}", e),
                })?;

                info!(medium_id = %e.medium_id, "MediumProjection: media record updated");
            }
        }

        Ok(())
    }
}
