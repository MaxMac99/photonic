use crate::{
    error::Result,
    user::{repo, repo::find_user_by_id, User, UserInput},
};
use sqlx::PgConnection;
use uuid::Uuid;

#[tracing::instrument(skip(conn))]
pub async fn create_or_update_user(conn: &mut PgConnection, user: UserInput) -> Result<()> {
    repo::create_or_update_user(conn, user).await
}

#[tracing::instrument(skip(conn))]
pub async fn get_user(conn: &mut PgConnection, user_id: Uuid) -> Result<User> {
    Ok(find_user_by_id(conn, user_id).await?)
}
