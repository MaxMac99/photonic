use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{
    common::StoreLocation,
    error::Result,
    medium_item::model::{MediumItem, MediumItemType},
};

pub async fn add_medium_item<'e, E>(
    executor: E,
    medium_id: &Uuid,
    item_type: MediumItemType,
    medium_item: MediumItem,
) -> Result<Uuid>
where
    E: PgExecutor<'e>,
{
    sqlx::query!("INSERT INTO medium_items (medium_id, medium_item_type, mime, filename, path, filesize, location, priority, timezone, taken_at, last_saved, width, height) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)\
        RETURNING id",
        medium_id,
        item_type as MediumItemType,
        medium_item.mime.to_string(),
        medium_item.filename,
        medium_item.path.into_os_string().into_string().unwrap(),
        medium_item.filesize as i64,
        medium_item.location as StoreLocation,
        medium_item.priority,
        medium_item.taken_at.timezone().local_minus_utc(),
        medium_item.taken_at,
        medium_item.last_saved,
        medium_item.width as i32,
        medium_item.height as i32)
        .fetch_one(&executor)
        .await
        .map(|item| item.id)
        .into()
}
