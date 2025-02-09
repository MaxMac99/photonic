use crate::{error::Result, medium::MediumItemType, state::ArcConnection};
use chrono::NaiveDateTime;
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
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[tracing::instrument(skip(conn))]
pub async fn create_medium_item(conn: &mut PgConnection, medium_item: MediumItemDb) -> Result<()> {
    sqlx::query!(
        "INSERT INTO medium_items (\
            id, \
            medium_id, \
            medium_item_type, \
            mime, \
            filename, \
            size, \
            priority, \
            width, \
            height) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        medium_item.id,
        medium_item.medium_id,
        medium_item.medium_item_type as MediumItemType,
        medium_item.mime,
        medium_item.filename,
        medium_item.size,
        medium_item.priority,
        medium_item.width,
        medium_item.height
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn find_medium_items_by_id(
    conn: ArcConnection<'_>,
    medium_id: Uuid,
) -> Result<Vec<MediumItemDb>> {
    let medium_items = sqlx::query_as!(
        MediumItemDb,
        "\
        SELECT \
            medium_items.id, \
            medium_id, \
            medium_item_type as \"medium_item_type: MediumItemType\", \
            mime, filename, \
            size, \
            priority, \
            width, \
            height, \
            last_saved, \
            deleted_at \
        FROM medium_items \
        WHERE medium_id = $1",
        medium_id
    )
    .fetch_all(conn.get_connection().await.as_mut())
    .await?;
    Ok(medium_items)
}

#[tracing::instrument(skip(conn))]
pub async fn find_medium_item_by_id(
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
) -> Result<Option<MediumItemDb>> {
    let medium_item = sqlx::query_as!(
        MediumItemDb,
        "\
        SELECT \
            id, \
            medium_id, \
            medium_item_type as \"medium_item_type: MediumItemType\", \
            mime, \
            filename,\
            size, \
            priority, \
            width, \
            height, \
            last_saved, \
            deleted_at \
        FROM medium_items \
        WHERE id = $1",
        medium_item_id
    )
    .fetch_optional(conn.get_connection().await.as_mut())
    .await?;
    Ok(medium_item)
}

#[tracing::instrument(skip(conn))]
pub async fn update_medium_item(conn: ArcConnection<'_>, medium_item: MediumItemDb) -> Result<()> {
    sqlx::query!(
        "UPDATE medium_items SET \
            medium_item_type = $1, \
            mime = $2, \
            filename = $3,\
            size = $4, \
            priority = $5, \
            width = $6, \
            height = $7 \
        WHERE id = $8",
        medium_item.medium_item_type as MediumItemType,
        medium_item.mime,
        medium_item.filename,
        medium_item.size,
        medium_item.priority,
        medium_item.width,
        medium_item.height,
        medium_item.id,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}
