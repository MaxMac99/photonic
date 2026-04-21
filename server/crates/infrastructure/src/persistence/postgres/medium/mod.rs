use application::medium::ports::{ExpiredTempLocation, MediumRepository};
use async_trait::async_trait;
use byte_unit::Byte;
use chrono::{DateTime, Utc};
use domain::{
    error::DomainResult,
    medium::{Medium, MediumFilter, MediumId, MediumListItem},
    user::UserId,
};
use sqlx::PgPool;

mod delete;
mod find_all;
mod find_by_id;
mod find_expired_temp;
mod save;
pub mod types;

pub struct PostgresMediumRepository {
    pool: PgPool,
}

impl PostgresMediumRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MediumRepository for PostgresMediumRepository {
    #[tracing::instrument(skip(self))]
    async fn find_by_id(&self, id: MediumId, user_id: UserId) -> DomainResult<Option<Medium>> {
        self.find_by_id_impl(id, user_id).await
    }

    #[tracing::instrument(skip(self))]
    async fn find_all(
        &self,
        filter: MediumFilter,
        user_id: UserId,
    ) -> DomainResult<Vec<MediumListItem>> {
        self.find_all_impl(filter, user_id).await
    }

    #[tracing::instrument(skip(self, medium), fields(medium_id = %medium.id, owner_id = %medium.owner_id, items_count = medium.items.len()))]
    async fn save(&self, medium: &Medium) -> DomainResult<()> {
        self.save_impl(medium).await
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: MediumId, user_id: UserId) -> DomainResult<()> {
        self.delete_impl(id, user_id).await
    }

    async fn get_user_usage(&self, user_id: UserId) -> DomainResult<Byte> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn find_expired_temp_locations(
        &self,
        created_before: DateTime<Utc>,
    ) -> DomainResult<Vec<ExpiredTempLocation>> {
        self.find_expired_temp_locations_impl(created_before).await
    }
}
