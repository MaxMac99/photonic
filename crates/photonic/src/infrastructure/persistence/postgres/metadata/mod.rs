mod delete;
mod entity;
mod find_by_id;
mod find_by_medium_id;
mod save;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    application::metadata::ports::MetadataRepository,
    domain::{
        error::DomainResult,
        medium::MediumId,
        metadata::{Metadata, MetadataId},
    },
};

pub struct PostgresMetadataRepository {
    pool: PgPool,
}

impl PostgresMetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MetadataRepository for PostgresMetadataRepository {
    #[tracing::instrument(skip(self), fields(metadata_id = %id))]
    async fn find_by_id(&self, id: MetadataId) -> DomainResult<Option<Metadata>> {
        self.find_by_id_impl(id).await
    }

    #[tracing::instrument(skip(self), fields(medium_id = %medium_id))]
    async fn find_by_medium_id(&self, medium_id: MediumId) -> DomainResult<Option<Metadata>> {
        self.find_by_medium_id_impl(medium_id).await
    }

    #[tracing::instrument(skip(self, metadata), fields(metadata_id = %metadata.id))]
    async fn save(&self, metadata: &Metadata) -> DomainResult<()> {
        self.save_impl(metadata).await
    }

    #[tracing::instrument(skip(self), fields(metadata_id = %id))]
    async fn delete(&self, id: MetadataId) -> DomainResult<()> {
        self.delete_impl(id).await
    }
}
