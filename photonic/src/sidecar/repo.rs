use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{common::StoreLocation, error::Result, sidecar::model::Sidecar};

pub async fn add_sidecar<'e, E>(executor: E, medium_id: &Uuid, file_item: Sidecar) -> Result<Uuid>
where
    E: PgExecutor<'e>,
{
    let query_result = sqlx::query!("INSERT INTO sidecars (medium_id, mime, filename, path, size, location, priority, last_saved) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)\
        RETURNING id",
        medium_id,
        file_item.mime.to_string(),
        file_item.filename,
        file_item.path.into_os_string().into_string().unwrap(),
        file_item.filesize as i64,
        file_item.location as StoreLocation,
        file_item.priority,
        file_item.last_saved)
        .fetch_one(&executor)
        .await?;
    Ok(query_result.id)
}
