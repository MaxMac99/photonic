use crate::{error::Result, state::ArcConnection};
use uuid::Uuid;

pub async fn add_tags(conn: ArcConnection<'_>, medium_id: Uuid, tags: Vec<String>) -> Result<()> {
    let medium_ids = vec![medium_id; tags.len()];
    sqlx::query!(
        "INSERT INTO media_tags (medium_id, tag_title) SELECT * FROM UNNEST ($1::uuid[], $2::text[])",
        &medium_ids[..],
        &tags[..],
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}
