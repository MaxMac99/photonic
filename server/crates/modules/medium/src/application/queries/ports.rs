use async_trait::async_trait;
use kernel::{error::DomainResult, MediumId, UserId};

use crate::domain::{MediumFilter, MediumListItem};

/// Read-side port for medium queries (ADR 0002): shaped read models from the
/// projection tables, one port per use-case family.
#[async_trait]
pub trait MediumQueryPort: Send + Sync {
    async fn find_all(
        &self,
        filter: MediumFilter,
        user_id: UserId,
    ) -> DomainResult<Vec<MediumListItem>>;

    async fn find_by_id(
        &self,
        medium_id: MediumId,
        user_id: UserId,
    ) -> DomainResult<Option<MediumListItem>>;
}
