use crate::{
    error::Result,
    state::ArcConnection,
    storage::{StorageLocation, StorageVariant},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct LocationDb {
    pub item_id: Uuid,
    pub path: String,
    pub variant: StorageVariant,
}

#[tracing::instrument(skip(conn))]
pub async fn add_storage_location(
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
    location: StorageLocation,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO locations (item_id, path, variant) VALUES ($1, $2, $3)",
        medium_item_id,
        location
            .path
            .into_os_string()
            .into_string()
            .expect("PathBuf to String conversion failed"),
        location.variant as StorageVariant,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn remove_location(
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
    location: StorageLocation,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM locations WHERE item_id = $1 AND variant = $2",
        medium_item_id,
        location.variant as StorageVariant,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn find_locations_by_medium_item_id(
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
) -> Result<Vec<StorageLocation>> {
    let locations = sqlx::query_as!(
        LocationDb,
        "SELECT item_id, path, variant as \"variant: StorageVariant\" FROM locations WHERE item_id = $1",
        medium_item_id,
    )
    .fetch_all(conn.get_connection().await.as_mut())
    .await?;
    Ok(locations
        .into_iter()
        .map(|location| StorageLocation {
            variant: location.variant,
            path: location.path.into(),
        })
        .collect())
}

#[tracing::instrument(skip(conn))]
pub async fn find_location_variant_by_medium_item_id(
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
    variant: StorageVariant,
) -> Result<Option<StorageLocation>> {
    let location = sqlx::query_as!(
        LocationDb,
        "SELECT item_id, path, variant as \"variant: StorageVariant\" FROM locations WHERE item_id = $1 AND variant = $2",
        medium_item_id,
        variant as StorageVariant,
    )
        .fetch_optional(conn.get_connection().await.as_mut())
        .await?;
    Ok(location.map(|location| StorageLocation {
        variant: location.variant,
        path: location.path.into(),
    }))
}
