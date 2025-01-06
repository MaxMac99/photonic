use crate::{
    error::Result,
    state::ArcConnection,
    user::{repo, repo::find_user_by_id, User, UserInput},
};
use uuid::Uuid;

#[tracing::instrument(skip(conn))]
pub async fn create_or_update_user(conn: ArcConnection<'_>, user: UserInput) -> Result<()> {
    repo::create_or_update_user(conn, user).await
}

#[tracing::instrument(skip(conn))]
pub async fn get_user(conn: ArcConnection<'_>, user_id: Uuid) -> Result<User> {
    Ok(find_user_by_id(conn, user_id).await?)
}
