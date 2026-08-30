use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::{error::DomainResult, MediumId, UserId};
use medium::{
    application::{
        ports::{ExpiredTempLocation, MediumRepository},
        queries::MediumQueryPort,
    },
    domain::{Medium, MediumFilter, MediumListItem},
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

/// Write-side port (ADR 0002).
#[async_trait]
impl MediumRepository for PostgresMediumRepository {
    #[tracing::instrument(skip(self))]
    async fn find_by_id(&self, id: MediumId, user_id: UserId) -> DomainResult<Option<Medium>> {
        self.find_by_id_impl(id, user_id).await
    }

    #[tracing::instrument(skip(self, medium), fields(medium_id = %medium.id, owner_id = %medium.owner_id, items_count = medium.items.len()))]
    async fn save(&self, medium: &Medium) -> DomainResult<()> {
        self.save_impl(medium).await
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: MediumId, user_id: UserId) -> DomainResult<()> {
        self.delete_impl(id, user_id).await
    }

    #[tracing::instrument(skip(self))]
    async fn find_expired_temp_locations(
        &self,
        created_before: DateTime<Utc>,
    ) -> DomainResult<Vec<ExpiredTempLocation>> {
        self.find_expired_temp_locations_impl(created_before).await
    }
}

/// Read-side port (ADR 0002): shaped read models from the projection tables.
#[async_trait]
impl MediumQueryPort for PostgresMediumRepository {
    #[tracing::instrument(skip(self))]
    async fn find_all(
        &self,
        filter: MediumFilter,
        user_id: UserId,
    ) -> DomainResult<Vec<MediumListItem>> {
        self.find_all_impl(filter, user_id).await
    }

    #[tracing::instrument(skip(self))]
    async fn find_by_id(
        &self,
        medium_id: MediumId,
        user_id: UserId,
    ) -> DomainResult<Option<MediumListItem>> {
        Ok(self
            .find_by_id_impl(medium_id, user_id)
            .await?
            .map(MediumListItem::from))
    }
}
