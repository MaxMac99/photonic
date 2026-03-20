use domain::{
    error::DomainResult,
    user::{User, UserId},
};
use tracing::debug;

use crate::persistence::postgres::{
    repo_error,
    user::{entity::UserDb, PostgresUserRepository},
};

impl PostgresUserRepository {
    pub(super) async fn find_by_id_impl(&self, id: UserId) -> DomainResult<Option<User>> {
        debug!("Querying user by id");

        let queried = sqlx::query_as!(
            UserDb,
            "SELECT id, version, username, email, quota, quota_used FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;

        match &queried {
            Some(_) => debug!("User found"),
            None => debug!("User not found"),
        }

        Ok(queried.map(Into::into))
    }
}
