use crate::{
    error::Result,
    medium::{model::MediumType, Direction, FindAllMediaOptions, MediumItemType},
    state::Transaction,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MediumDb {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub leading_item_id: Uuid,
}

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
pub struct MediumItemInfoDb {
    pub id: Uuid,
    pub taken_at: Option<DateTime<Utc>>,
    pub taken_at_timezone: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
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
    pub width: Option<i32>,
    pub height: Option<i32>,
}

pub async fn create_medium(transaction: &mut Transaction, medium: MediumDb) -> Result<()> {
    sqlx::query!(
        "INSERT INTO media (id, owner_id, medium_type, leading_item_id) \
        VALUES ($1, $2, $3, $4)",
        medium.id,
        medium.owner_id,
        medium.medium_type as MediumType,
        medium.leading_item_id,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn create_medium_item(
    transaction: &mut Transaction,
    medium_item: MediumItemDb,
) -> Result<()> {
    sqlx::query!("INSERT INTO medium_items (id, medium_id, medium_item_type, mime, filename, size, priority) \
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
        medium_item.id,
        medium_item.medium_id,
        medium_item.medium_item_type as MediumItemType,
        medium_item.mime,
        medium_item.filename,
        medium_item.size,
        medium_item.priority)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub async fn create_medium_item_info(
    transaction: &mut Transaction,
    medium_item_info: MediumItemInfoDb,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO medium_item_info (id, taken_at, taken_at_timezone) VALUES ($1, $2, $3)",
        medium_item_info.id,
        medium_item_info.taken_at,
        medium_item_info.taken_at_timezone,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn find_media(
    transaction: &mut Transaction,
    owner_id: Uuid,
    filter: FindAllMediaOptions,
) -> Result<Vec<MediumDb>> {
    let mut query = QueryBuilder::new(
        "\
        SELECT media.id, media.owner_id, media.medium_type, media.leading_item_id \
        FROM media \
        JOIN medium_item_info ON media.leading_item_id = medium_item_info.id \
        WHERE media.owner_id =
        ",
    );
    query.push_bind(owner_id);
    if let Some(start_date) = filter.start_date {
        query.push(" AND medium_item_info.taken_at >= ");
        query.push_bind(start_date);
    }
    if let Some(end_date) = filter.end_date {
        query.push(" AND medium_item_info.taken_at <= ");
        query.push_bind(end_date);
    }
    if let Some(page_last_date) = filter.page_last_date {
        query.push(" AND medium_item_info.taken_at ");
        query.push(match filter.direction {
            Direction::Asc => " > ",
            Direction::Desc => " < ",
        });
        query.push_bind(page_last_date);
    }
    if let Some(page_last_id) = filter.page_last_id {
        query.push(" AND media.id < ");
        query.push_bind(page_last_id);
    }

    query.push(" ORDER BY medium_item_info.taken_at ");
    query.push(filter.direction.to_sql());
    query.push(", media.id ");
    query.push(filter.direction.to_sql());
    query.push(" LIMIT ");
    query.push_bind(filter.per_page as i64);

    let media = query
        .build_query_as::<MediumDb>()
        .fetch_all(&mut **transaction)
        .await?;
    Ok(media)
}

pub async fn find_medium_items_by_id(
    transaction: &mut Transaction,
    medium_id: Uuid,
) -> Result<Vec<FullMediumItemDb>> {
    let medium_items = sqlx::query_as!(FullMediumItemDb, "\
        SELECT medium_items.id, medium_id, medium_item_type as \"medium_item_type: MediumItemType\", mime, filename, size, priority, last_saved, deleted_at, taken_at, taken_at_timezone, width, height \
        FROM medium_items \
        JOIN medium_item_info \
        ON medium_items.id = medium_item_info.id \
        WHERE medium_id = $1", medium_id)
        .fetch_all(&mut **transaction)
        .await?;
    Ok(medium_items)
}
