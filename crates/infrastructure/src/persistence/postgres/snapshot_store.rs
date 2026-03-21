use std::marker::PhantomData;

use application::{error::ApplicationResult, snapshot_store::SnapshotStore};
use async_trait::async_trait;
use domain::aggregate::{AggregateRoot, AggregateVersion};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{PgPool, Row};

use super::repo_error;

pub struct PostgresSnapshotStore<A: AggregateRoot> {
    pool: PgPool,
    _phantom: PhantomData<A>,
}

impl<A: AggregateRoot> PostgresSnapshotStore<A> {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<A> SnapshotStore<A> for PostgresSnapshotStore<A>
where
    A: AggregateRoot + Serialize + DeserializeOwned + 'static,
{
    async fn load_snapshot(
        &self,
        aggregate_id: &str,
    ) -> ApplicationResult<Option<(A, AggregateVersion)>> {
        let stream_id = format!("{}-{}", A::aggregate_type(), aggregate_id);

        let row = sqlx::query("SELECT payload, version FROM snapshots WHERE stream_id = $1")
            .bind(&stream_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| application::error::ApplicationError::Domain {
                source: repo_error(e),
            })?;

        match row {
            Some(row) => {
                let payload: serde_json::Value = row.get("payload");
                let version: i64 = row.get("version");
                let aggregate: A =
                    serde_json::from_value(payload).map_err(|e| {
                        application::error::ApplicationError::Internal {
                            message: format!("Failed to deserialize snapshot: {}", e),
                        }
                    })?;
                Ok(Some((aggregate, version)))
            }
            None => Ok(None),
        }
    }

    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        aggregate: &A,
        version: AggregateVersion,
    ) -> ApplicationResult<()> {
        let stream_id = format!("{}-{}", A::aggregate_type(), aggregate_id);
        let payload = serde_json::to_value(aggregate).map_err(|e| {
            application::error::ApplicationError::Internal {
                message: format!("Failed to serialize snapshot: {}", e),
            }
        })?;

        sqlx::query(
            "INSERT INTO snapshots (stream_id, version, payload, created_at)
             VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
             ON CONFLICT (stream_id)
             DO UPDATE SET version = $2, payload = $3, created_at = CURRENT_TIMESTAMP",
        )
        .bind(&stream_id)
        .bind(version)
        .bind(&payload)
        .execute(&self.pool)
        .await
        .map_err(|e| application::error::ApplicationError::Domain {
            source: repo_error(e),
        })?;

        Ok(())
    }
}
