use crate::medium::model::MediumType;
use common::{error::Result, medium::MediumType};
use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn create_medium<'e, E>(
    executor: E,
    owner: &Uuid,
    medium_type: MediumType,
) -> Result<Uuid>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query!(
        "INSERT INTO media (owner_id, medium_type) \
        VALUES ($1, $2)\
        RETURNING id",
        owner,
        medium_type as MediumType,
    )
    .fetch_one(executor)
    .await
    .map(|record| record.id)?)
}
