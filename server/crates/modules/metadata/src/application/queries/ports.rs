use async_trait::async_trait;
use kernel::{error::DomainResult, MediumId, UserId};

use crate::domain::Metadata;

/// Read-side port for metadata queries (ADR 0002), ownership-scoped
/// per ADR 0008.
#[async_trait]
pub trait MetadataQueryPort: Send + Sync {
    async fn find_by_medium_id(
        &self,
        medium_id: MediumId,
        user_id: UserId,
    ) -> DomainResult<Option<Metadata>>;
}
