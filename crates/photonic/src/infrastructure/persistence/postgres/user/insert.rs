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
    pub(super) async fn insert_impl(&self, user: &User) -> DomainResult<()> {
        debug!("Inserting new user into database");

        let result = sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, quota, quota_used, version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
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
            version = user.version,
            rows_affected = result.rows_affected(),
            "User inserted successfully"
        );

        Ok(())
    }
}
