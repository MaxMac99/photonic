use crate::{
    error::Result,
    state::ArcConnection,
    user::{User, UserInput},
};
use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct UserDb {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub quota: i64,
    pub quota_used: i64,
}

impl Into<User> for UserDb {
    fn into(self) -> User {
        User {
            id: self.id,
            username: self.username.unwrap_or_else(|| "".to_string()),
            email: self.email,
            quota: Byte::from_u64(self.quota as u64),
            quota_used: Byte::from_u64(self.quota_used as u64),
        }
    }
}

#[tracing::instrument(skip(conn))]
pub async fn find_user_by_id(conn: ArcConnection<'_>, id: Uuid) -> Result<User> {
    let queried = sqlx::query_as!(
        UserDb,
        "SELECT id, username, email, quota, quota_used FROM users WHERE id = $1",
        id
    )
    .fetch_one(conn.get_connection().await.as_mut())
    .await?;
    Ok(queried.into())
}

#[tracing::instrument(skip(conn))]
pub async fn create_or_update_user(conn: ArcConnection<'_>, user: UserInput) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users (id, username, email, quota, quota_used)\
        VALUES ($1, $2, $3, $4, 0)\
        ON CONFLICT (id)\
        DO UPDATE SET username = $2, email = $3, quota = $4;",
        user.sub,
        user.get_username(),
        user.email,
        user.quota.as_u64() as i64,
    )
    .execute(conn.get_connection().await.as_mut())
    .await?;
    Ok(())
}
