use crate::{
    album::model::FindAllAlbumsOptions, common::Direction, error::Result, state::ArcConnection,
};
use chrono::{DateTime, Utc};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InsertAlbumDb {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SelectAlbumDb {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub number_of_items: i64,
    pub minimum_date: Option<DateTime<Utc>>,
    pub maximum_date: Option<DateTime<Utc>>,
}

#[tracing::instrument(skip(conn))]
pub async fn create_album(conn: ArcConnection<'_>, album: InsertAlbumDb) -> Result<()> {
    sqlx::query!(
        "INSERT INTO albums (id, owner_id, title, description) \
        VALUES ($1, $2, $3, $4)",
        album.id,
        album.owner_id,
        album.title,
        album.description,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn get_album(conn: ArcConnection<'_>, album_id: Uuid) -> Result<Option<InsertAlbumDb>> {
    let album = sqlx::query_as!(
        InsertAlbumDb,
        "SELECT id, owner_id, title, description FROM albums WHERE id = $1",
        album_id
    )
    .fetch_optional(conn.get_connection().await.as_mut())
    .await?;
    Ok(album)
}

#[tracing::instrument(skip(conn))]
pub async fn find_albums(
    conn: ArcConnection<'_>,
    owner_id: Uuid,
    filter: FindAllAlbumsOptions,
) -> Result<Vec<SelectAlbumDb>> {
    let mut query = QueryBuilder::new(
        "\
        SELECT \
            album.id, \
            album.title, \
            album.description, \
            COUNT(media.id) as number_of_items, \
            MIN(medium_item_info.taken_at) as minimum_date, \
            MAX(medium_item_info.taken_at) as maximum_date, \
        FROM album \
        ",
    );

    if filter.include_empty_albums {
        query.push(
            "LEFT JOIN media ON album.id = media.album_id \
        LEFT JOIN medium_item_info ON media.leading_item_id = media_item_info.id ",
        );
    } else {
        query.push(
            "JOIN media ON album.id = media.album_id \
        JOIN medium_item_info ON media.leading_item_id = media_item_info.id ",
        );
    }

    query.push(" WHERE album.owner_id = ");
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
        query.push(" AND album.id ");
        query.push(match filter.direction {
            Direction::Asc => " > ",
            Direction::Desc => " < ",
        });
        query.push_bind(page_last_id);
    }

    query.push(" ORDER BY medium_item_info.taken_at ");
    query.push(filter.direction.to_sql());
    query.push(", album.id ");
    query.push(filter.direction.to_sql());
    query.push(" LIMIT ");
    query.push_bind(filter.per_page as i64);

    let albums = query
        .build_query_as::<SelectAlbumDb>()
        .fetch_all(conn.get_connection().await.as_mut())
        .await?;
    Ok(albums)
}
