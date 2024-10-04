use chrono::{DateTime, Utc};
use sqlx::{Execute, PgExecutor, QueryBuilder};
use tracing::debug;
use uuid::Uuid;

use crate::{
    common::DateDirection,
    error::{Error, FindMediumByIdSnafu, Result},
    medium::model::{MediumDb, MediumType},
};

pub async fn create_medium<'e, E>(
    executor: E,
    owner: &Uuid,
    medium_type: MediumType,
    album_id: Option<Uuid>,
) -> Result<Uuid>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        "INSERT INTO medium (owner_id, medium_type, album_id) \
        VALUES ($1, $2, $3)\
        RETURNING id",
        owner,
        medium_type as MediumType,
        album_id
    )
    .fetch_one(executor)
    .await
    .map(|record| record.id)
    .into()
}

pub async fn find_media<'e, E>(
    executor: E,
    user_id: Uuid,
    page_size: i64,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    album_id: Option<Uuid>,
    show_only_unset_albums: bool,
    date_direction: DateDirection,
) -> Result<Vec<MediumDb>>
where
    E: PgExecutor<'e>,
{
    // Join first match only of medium_items with the newest date
    let mut query_builder = QueryBuilder::new(
        "SELECT * FROM medium m\
            JOIN (\
                SELECT DISTINCT ON (mis.medium_id) * \
                FROM medium_items mis \
                ORDER BY mis.medium_id, mis.taken_at DESC\
            ) AS mi \
            ON mi.medium_id = m.id \
            WHERE m.owner_id = ",
    )
    .push_bind(user_id);

    if let Some(start_date) = start_date {
        query_builder.push(" WHERE mi.taken_at >= ");
        query_builder.push_bind(start_date);
    }
    if let Some(end_date) = end_date {
        query_builder.push(" WHERE mi.taken_at <= ");
        query_builder.push_bind(end_date);
    }
    if let Some(album_id) = album_id {
        query_builder.push(" WHERE mi.album_id = ");
        query_builder.push_bind(album_id);
    } else if show_only_unset_albums {
        query_builder.push(" WHERE m.album_id IS NULL");
    }
    match date_direction {
        DateDirection::NewestFirst => query_builder.push(" ORDER BY mi.taken_at DESC"),
        DateDirection::OldestFirst => query_builder.push(" ORDER BY mi.taken_at ASC"),
    };

    query_builder.push(" LIMIT ");
    query_builder.push_bind(page_size);

    let mut query = query_builder.build_query_as::<MediumDb>();
    debug!("Query: {:?}", query.sql());
    query.fetch_all(executor)
}

pub async fn get_medium<'e, E>(executor: E, id: Uuid, user_id: Uuid) -> Result<MediumDb>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Medium,
        "SELECT id, owner_id, medium_type as \"medium_type: MediumType\", album_id, deleted_at \
        FROM medium \
        WHERE id = $1 AND owner_id = $2",
        id,
        user_id
    )
    .fetch_one(executor)
    .await
    .into()
}
