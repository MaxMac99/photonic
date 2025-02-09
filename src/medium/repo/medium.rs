use crate::{
    common::Direction,
    error::Result,
    medium::{FindAllMediaOptions, MediumType},
    state::ArcConnection,
};
use chrono::{DateTime, Utc};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MediumDb {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub leading_item_id: Uuid,
    pub album_id: Option<Uuid>,
    pub taken_at: Option<DateTime<Utc>>,
    pub taken_at_timezone: Option<i32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

#[tracing::instrument(skip(conn))]
pub async fn create_medium(conn: ArcConnection<'_>, medium: MediumDb) -> Result<()> {
    sqlx::query!(
        "INSERT INTO media (\
            id, \
            owner_id, \
            medium_type, \
            leading_item_id, \
            album_id,\
            taken_at, \
            taken_at_timezone,\
            camera_make,\
            camera_model) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        medium.id,
        medium.owner_id,
        medium.medium_type as MediumType,
        medium.leading_item_id,
        medium.album_id,
        medium.taken_at,
        medium.taken_at_timezone,
        medium.camera_make,
        medium.camera_model,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn find_medium(conn: ArcConnection<'_>, medium_id: Uuid) -> Result<Option<MediumDb>> {
    let medium = sqlx::query_as!(
        MediumDb,
        "SELECT \
            id, \
            owner_id, \
            medium_type as \"medium_type: MediumType\", \
            leading_item_id, \
            album_id, \
            taken_at, \
            taken_at_timezone, \
            camera_make, \
            camera_model \
        FROM media \
        WHERE id = $1",
        medium_id,
    )
    .fetch_optional(conn.get_connection().await.as_mut())
    .await?;
    Ok(medium)
}

#[tracing::instrument(skip(conn))]
pub async fn get_media(
    conn: ArcConnection<'_>,
    owner_id: Uuid,
    filter: FindAllMediaOptions,
) -> Result<Vec<MediumDb>> {
    let mut query = QueryBuilder::new(
        "\
        SELECT \
            media.id, \
            media.owner_id, \
            media.medium_type, \
            media.leading_item_id, \
            media.album_id, \
            media.taken_at, \
            media.taken_at_timezone, \
            media.camera_make, \
            media.camera_model \
        FROM media",
    );
    if !filter.tags.is_empty() {
        query.push("JOIN media_tags ON media.id = media_tags.medium_id");
    }
    query.push(" WHERE media.owner_id = ");
    query.push_bind(owner_id);
    if let Some(start_date) = filter.start_date {
        query.push(" AND media.taken_at >= ");
        query.push_bind(start_date);
    }
    if let Some(end_date) = filter.end_date {
        query.push(" AND media.taken_at <= ");
        query.push_bind(end_date);
    }
    if let Some(album_id) = filter.album_id {
        if filter.include_no_album {
            query.push(" AND (media.album_id = ");
            query.push_bind(album_id);
            query.push(" OR media.album_id IS NULL)");
        } else {
            query.push(" AND media.album_id = ");
            query.push_bind(album_id);
        }
    }
    if !filter.tags.is_empty() {
        query.push(" AND media_tags.tag_title = ANY (");
        filter.tags.into_iter().for_each(|tag| {
            query.push_bind(tag);
            query.push(", ");
        });
        query.push(")");
    }
    if let Some(page_last_date) = filter.page_last_date {
        query.push(" AND media.taken_at ");
        query.push(match filter.direction {
            Direction::Asc => " > ",
            Direction::Desc => " < ",
        });
        query.push_bind(page_last_date);
    }
    if let Some(page_last_id) = filter.page_last_id {
        query.push(" AND media.id ");
        query.push(match filter.direction {
            Direction::Asc => " > ",
            Direction::Desc => " < ",
        });
        query.push_bind(page_last_id);
    }

    query.push(" ORDER BY media.taken_at ");
    query.push(filter.direction.to_sql());
    query.push(", media.id ");
    query.push(filter.direction.to_sql());
    query.push(" LIMIT ");
    query.push_bind(filter.per_page as i64);

    let media = query
        .build_query_as::<MediumDb>()
        .fetch_all(conn.get_connection().await.as_mut())
        .await?;
    Ok(media)
}

#[tracing::instrument(skip(conn))]
pub async fn update_medium(conn: ArcConnection<'_>, medium: MediumDb) -> Result<()> {
    sqlx::query!(
        "UPDATE media SET \
            owner_id = $1, \
            medium_type = $2, \
            leading_item_id = $3, \
            album_id = $4, \
            taken_at = $5, \
            taken_at_timezone = $6, \
            camera_make = $7, \
            camera_model = $8 \
        WHERE id = $9",
        medium.owner_id,
        medium.medium_type as MediumType,
        medium.leading_item_id,
        medium.album_id,
        medium.taken_at,
        medium.taken_at_timezone,
        medium.camera_make,
        medium.camera_model,
        medium.id,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn delete_medium(conn: ArcConnection<'_>, owner_id: Uuid, medium_id: Uuid) -> Result<()> {
    sqlx::query!(
        "DELETE FROM media WHERE owner_id = $1 AND id = $2",
        owner_id,
        medium_id,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}
