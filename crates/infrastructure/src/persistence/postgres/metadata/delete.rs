use domain::{error::DomainResult, metadata::MetadataId};

use super::PostgresMetadataRepository;
use crate::persistence::postgres::repo_error;

impl PostgresMetadataRepository {
    pub(super) async fn delete_impl(&self, id: MetadataId) -> DomainResult<()> {
        sqlx::query("DELETE FROM metadata WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(repo_error)?;

        Ok(())
    }
}
