use crate::{
    error::Result,
    medium::{Direction, FindAllMediaOptions, MediumType},
};
use sqlx::{PgConnection, QueryBuilder};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MediumDb {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub leading_item_id: Uuid,
}

#[tracing::instrument(skip(conn))]
pub async fn create_medium(conn: &mut PgConnection, medium: MediumDb) -> Result<()> {
    sqlx::query!(
        "INSERT INTO media (id, owner_id, medium_type, leading_item_id) \
        VALUES ($1, $2, $3, $4)",
        medium.id,
        medium.owner_id,
        medium.medium_type as MediumType,
        medium.leading_item_id,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn find_media(
    conn: &mut PgConnection,
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
        .fetch_all(&mut *conn)
        .await?;
    Ok(media)
}

#[tracing::instrument(skip(conn))]
pub async fn delete_medium(conn: &mut PgConnection, owner_id: Uuid, medium_id: Uuid) -> Result<()> {
    sqlx::query!(
        "DELETE FROM media WHERE owner_id = $1 AND id = $2",
        owner_id,
        medium_id,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
