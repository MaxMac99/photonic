use tracing::{debug, info};

use crate::{
    domain::{error::DomainResult, medium::MediumId, user::UserId},
    infrastructure::persistence::postgres::medium::PostgresMediumRepository,
};

impl PostgresMediumRepository {
    pub(super) async fn delete_impl(&self, id: MediumId, user_id: UserId) -> DomainResult<()> {
        debug!("Deleting medium from database");

        let result = sqlx::query!(
            "DELETE FROM media WHERE owner_id = $1 AND id = $2",
            user_id,
            id,
        )
        .execute(&self.pool)
        .await?;

        info!(
            rows_affected = result.rows_affected(),
            "Medium deleted successfully"
        );

        Ok(())
    }
}
