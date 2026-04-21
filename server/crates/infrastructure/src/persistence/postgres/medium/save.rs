use chrono::Utc;
use domain::{error::DomainResult, medium::Medium};
use tracing::{debug, info};

use crate::persistence::postgres::{
    medium::{
        types::{MediumItemTypeDb, MediumTypeDb, StorageTierDb},
        PostgresMediumRepository,
    },
    repo_error,
};

impl PostgresMediumRepository {
    pub(super) async fn save_impl(&self, medium: &Medium) -> DomainResult<()> {
        debug!("Saving medium aggregate to database");

        // Start a transaction for the whole aggregate
        let mut tx = self.pool.begin().await.map_err(repo_error)?;

        // Extract taken_at components
        let (taken_at_utc, taken_at_timezone) = medium
            .taken_at
            .as_ref()
            .map(|dt| {
                (
                    dt.with_timezone(&chrono::Utc),
                    dt.offset().local_minus_utc(),
                )
            })
            .unzip();

        let (gps_lat, gps_lng, gps_alt) = medium
            .gps_coordinates
            .map(|gps| (Some(gps.latitude()), Some(gps.longitude()), gps.altitude()))
            .unwrap_or((None, None, None));

        let updated_at = Utc::now().naive_utc();

        // UPSERT media record
        sqlx::query!(
            r#"
            INSERT INTO media (
                id,
                owner_id,
                medium_type,
                leading_item_id,
                taken_at,
                taken_at_timezone,
                camera_make,
                camera_model,
                gps_latitude,
                gps_longitude,
                gps_altitude,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE
            SET medium_type = EXCLUDED.medium_type,
                leading_item_id = EXCLUDED.leading_item_id,
                taken_at = EXCLUDED.taken_at,
                taken_at_timezone = EXCLUDED.taken_at_timezone,
                camera_make = EXCLUDED.camera_make,
                camera_model = EXCLUDED.camera_model,
                gps_latitude = EXCLUDED.gps_latitude,
                gps_longitude = EXCLUDED.gps_longitude,
                gps_altitude = EXCLUDED.gps_altitude,
                updated_at = EXCLUDED.updated_at
            "#,
            medium.id,
            medium.owner_id,
            MediumTypeDb::from(medium.medium_type) as MediumTypeDb,
            medium.leading_item_id,
            taken_at_utc,
            taken_at_timezone,
            medium.camera_make,
            medium.camera_model,
            gps_lat,
            gps_lng,
            gps_alt,
            updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;

        debug!(medium_id = %medium.id, "Media record saved");

        // Save each medium item
        for item in &medium.items {
            // UPSERT medium_item
            sqlx::query!(
                r#"
                INSERT INTO medium_items (
                    id,
                    medium_id,
                    medium_item_type,
                    mime,
                    filename,
                    size,
                    priority,
                    width,
                    height,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (id) DO UPDATE
                SET medium_item_type = EXCLUDED.medium_item_type,
                    mime = EXCLUDED.mime,
                    filename = EXCLUDED.filename,
                    size = EXCLUDED.size,
                    priority = EXCLUDED.priority,
                    width = EXCLUDED.width,
                    height = EXCLUDED.height,
                    updated_at = EXCLUDED.updated_at
                "#,
                item.id,
                item.medium_id,
                MediumItemTypeDb::from(item.medium_item_type) as MediumItemTypeDb,
                item.mime.to_string(),
                item.filename.as_str(),
                item.filesize.as_u64() as i64,
                item.priority.value() as i32,
                item.dimensions.as_ref().map(|d| d.width() as i32),
                item.dimensions.as_ref().map(|d| d.height() as i32),
                updated_at
            )
            .execute(&mut *tx)
            .await
            .map_err(repo_error)?;

            debug!(item_id = %item.id, "Medium item saved");

            // Delete old locations for this item
            sqlx::query!("DELETE FROM locations WHERE item_id = $1", item.id,)
                .execute(&mut *tx)
                .await
                .map_err(repo_error)?;

            // Insert the file location
            for location in &item.locations {
                sqlx::query!(
                    r#"
                    INSERT INTO locations (item_id, path, variant)
                    VALUES ($1, $2, $3)
                    "#,
                    item.id,
                    location.relative_path.to_str().unwrap_or(""),
                    StorageTierDb::from(location.storage_tier.clone()) as StorageTierDb,
                )
                .execute(&mut *tx)
                .await
                .map_err(repo_error)?;

                debug!(item_id = %item.id, storage_tier = ?location.storage_tier, "Location saved");
            }
        }

        // Commit the transaction
        tx.commit().await.map_err(repo_error)?;

        info!(
            medium_id = %medium.id,
            items_saved = medium.items.len(),
            "Medium aggregate saved successfully"
        );

        Ok(())
    }
}
