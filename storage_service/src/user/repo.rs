use crate::user::service::CreateUserInput;
use common::error::Result;
use sqlx::PgExecutor;

pub async fn create_or_update_user<'e, E>(executor: E, user: CreateUserInput) -> Result<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        "INSERT INTO users (id, username, email, quota, quota_used)\
        VALUES ($1, $2, $3, $4, 0)\
        ON CONFLICT (id)\
        DO UPDATE SET username = $2, email = $3, quota = $4;",
        user.id,
        user.username,
        user.email,
        user.quota.as_u64() as i64,
    )
    .execute(executor)
    .await?;
    Ok(())
}
