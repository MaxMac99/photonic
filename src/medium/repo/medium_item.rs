use crate::{error::Result, medium::MediumItemType};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct MediumItemDb {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: String,
    pub filename: String,
    pub size: i64,
    pub priority: i32,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct FullMediumItemDb {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: String,
    pub filename: String,
    pub size: i64,
    pub priority: i32,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub taken_at: Option<DateTime<Utc>>,
    pub taken_at_timezone: Option<i32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[tracing::instrument(skip(conn))]
pub async fn create_medium_item(conn: &mut PgConnection, medium_item: MediumItemDb) -> Result<()> {
    sqlx::query!("INSERT INTO medium_items (id, medium_id, medium_item_type, mime, filename, size, priority) \
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
        medium_item.id,
        medium_item.medium_id,
        medium_item.medium_item_type as MediumItemType,
        medium_item.mime,
        medium_item.filename,
        medium_item.size,
        medium_item.priority)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn find_medium_items_by_id(
    conn: &mut PgConnection,
    medium_id: Uuid,
) -> Result<Vec<FullMediumItemDb>> {
    let medium_items = sqlx::query_as!(FullMediumItemDb, "\
        SELECT medium_items.id, medium_id, medium_item_type as \"medium_item_type: MediumItemType\", mime, filename, size, priority, last_saved, deleted_at, taken_at, taken_at_timezone, camera_make, camera_model, width, height \
        FROM medium_items \
        JOIN medium_item_info \
        ON medium_items.id = medium_item_info.id \
        WHERE medium_id = $1", medium_id)
        .fetch_all(&mut *conn)
        .await?;
    Ok(medium_items)
}
