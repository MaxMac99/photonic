use crate::{error::Result, user::service::CreateUserInput};
use sqlx::PgExecutor;

pub async fn create_or_update_user<'e, E>(executor: E, user: CreateUserInput) -> Result<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        "INSERT INTO users (id, username, email, given_name, quota, quota_used)\
        VALUES ($1, $2, $3, $4, $5, 0)\
        ON CONFLICT (id)\
        DO UPDATE SET username = $2, email = $3, given_name = $4, quota = $5;",
        user.id,
        user.username,
        user.email,
        user.given_name,
        user.quota as i64
    )
    .execute(executor)
    .await?;
    Ok(())
}
