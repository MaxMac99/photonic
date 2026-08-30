mod delete;
pub(crate) mod entity;
mod find_by_medium_id;
mod save;

use async_trait::async_trait;
use kernel::{error::DomainResult, MetadataId};
use metadata::{application::ports::MetadataRepository, domain::Metadata};
use sqlx::PgPool;

pub struct PostgresMetadataRepository {
    pool: PgPool,
}

impl PostgresMetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Write-side port (ADR 0002).
#[async_trait]
impl MetadataRepository for PostgresMetadataRepository {
    #[tracing::instrument(skip(self, metadata), fields(metadata_id = %metadata.id))]
    async fn save(&self, metadata: &Metadata) -> DomainResult<()> {
        self.save_impl(metadata).await
    }

    #[tracing::instrument(skip(self), fields(metadata_id = %id))]
    async fn delete(&self, id: MetadataId) -> DomainResult<()> {
        self.delete_impl(id).await
    }
}
