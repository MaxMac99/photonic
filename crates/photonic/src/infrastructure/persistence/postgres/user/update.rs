use snafu::ensure;
use tracing::{debug, info};

use crate::{
    domain::{
        error::{ConcurrentModificationSnafu, DomainResult},
        user::User,
    },
    infrastructure::persistence::postgres::user::PostgresUserRepository,
};

impl PostgresUserRepository {
    pub(super) async fn update_impl(&self, user: &User) -> DomainResult<()> {
        debug!("Updating existing user in database");

        let result = sqlx::query!(
            r#"
            UPDATE users
            SET username = $2,
                email = $3,
                quota = $4,
                quota_used = $5,
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1 AND version = $6
            "#,
            user.id,
            user.username,
            user.email,
            user.quota.limit().as_u64() as i64,
            user.quota.used().as_u64() as i64,
            user.version
        )
        .execute(&self.pool)
        .await?;

        ensure!(
            result.rows_affected() > 0,
            ConcurrentModificationSnafu {
                aggregate_id: user.id,
                expected_version: user.version,
            }
        );

        info!(
            new_version = user.version + 1,
            rows_affected = result.rows_affected(),
            "User updated successfully"
        );

        Ok(())
    }
}
