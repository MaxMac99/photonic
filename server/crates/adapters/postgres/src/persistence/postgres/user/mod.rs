mod entity;
mod find_by_id;
mod insert;
mod update;

use async_trait::async_trait;
use kernel::{error::DomainResult, UserId};
use sqlx::PgPool;
use user::{application::ports::UserRepository, domain::User};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    #[tracing::instrument(skip(self), fields(user_id = %id))]
    async fn find_by_id(&self, id: UserId) -> DomainResult<Option<User>> {
        self.find_by_id_impl(id).await
    }

    #[tracing::instrument(skip(self, user), fields(user_id = %user.id, version = user.version))]
    async fn insert(&self, user: &User) -> DomainResult<()> {
        self.insert_impl(user).await
    }

    #[tracing::instrument(skip(self, user), fields(user_id = %user.id, version = user.version))]
    async fn update(&self, user: &User) -> DomainResult<()> {
        self.update_impl(user).await
    }
}
