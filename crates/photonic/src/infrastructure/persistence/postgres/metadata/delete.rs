use super::PostgresMetadataRepository;
use crate::domain::{error::DomainResult, metadata::MetadataId};

impl PostgresMetadataRepository {
    pub(super) async fn delete_impl(&self, id: MetadataId) -> DomainResult<()> {
        sqlx::query("DELETE FROM metadata WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
