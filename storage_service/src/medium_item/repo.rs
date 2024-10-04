use crate::medium_item::model::{MediumItem, MediumItemType};
use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use common::{
    error::Result,
    stream::events::{StorageLocation, StorageVariant},
};
use mime_serde_shim::Wrapper;
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub(super) struct MediumItemDb {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: String,
    pub filename: String,
    pub variant: StorageVariant,
    pub path: PathBuf,
    pub size: i64,
    pub priority: i32,
    pub taken_at: Option<DateTime<Utc>>,
    pub taken_at_timezone: Option<i32>,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl Into<MediumItem> for MediumItemDb {
    fn into(self) -> MediumItem {
        MediumItem {
            id: self.id,
            medium_id: self.medium_id,
            medium_item_type: self.medium_item_type,
            mime: Wrapper(
                self.mime
                    .parse()
                    .expect("Database entry is not of type mime"),
            ),
            filename: self.filename,
            location: StorageLocation {
                variant: self.variant,
                path: self.path,
            },
            filesize: Byte::from_u64(self.size as u64),
            priority: self.priority,
            taken_at: self.taken_at.map(|date| {
                date.with_timezone(
                    &FixedOffset::east_opt(self.taken_at_timezone.unwrap_or(0))
                        .expect("Database entry has wrong timezone"),
                )
            }),
            last_saved: self.last_saved,
            deleted_at: self.deleted_at,
            width: self.width,
            height: self.height,
        }
    }
}

pub async fn add_medium_item<'e, E>(executor: E, medium_item: MediumItem) -> Result<Uuid>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query!("INSERT INTO medium_items (medium_id, medium_item_type, mime, filename, path, size, variant, priority, taken_at, taken_at_timezone, last_saved, width, height) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)\
        RETURNING id",
        medium_item.medium_id,
        medium_item.medium_item_type as MediumItemType,
        medium_item.mime.to_string(),
        medium_item.filename,
        medium_item.location.path.into_os_string().into_string().unwrap(),
        medium_item.filesize.as_u64() as i64,
        medium_item.location.variant as StorageVariant,
        medium_item.priority,
        medium_item.taken_at,
        medium_item.taken_at.map(|date| date.timezone().local_minus_utc()),
        medium_item.last_saved,
        medium_item.width,
        medium_item.height)
        .fetch_one(executor)
        .await
        .map(|item| item.id)?)
}

pub async fn find_medium_item_by_id<'e, E>(executor: E, id: Uuid) -> Result<MediumItem>
where
    E: PgExecutor<'e>,
{
    let queried = sqlx::query_as!(MediumItemDb, "SELECT id, medium_id, medium_item_type as \"medium_item_type: MediumItemType\", mime, filename, variant as \"variant: StorageVariant\", path, size, priority, taken_at, taken_at_timezone, last_saved, deleted_at, width, height FROM medium_items WHERE id = $1", id)
        .fetch_one(executor)
        .await?;
    Ok(queried.into())
}
