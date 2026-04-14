use async_trait::async_trait;
use event_sourcing::aggregate::snapshot_store::{Snapshot, SnapshotStore};
use event_sourcing::aggregate::traits::Aggregate;
use event_sourcing::error::{EventSourcingError, Result};
use event_sourcing::stream::stream_id::StreamId;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{PgPool, Row};

/// Postgres implementation of `SnapshotStore<A, i64>`.
///
/// Stores aggregate snapshots in the `snapshots` table as JSONB payloads.
/// The aggregate must implement `Serialize + DeserializeOwned`.
pub struct PostgresSnapshotStore {
    pool: PgPool,
}

impl PostgresSnapshotStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<A> SnapshotStore<A, i64> for PostgresSnapshotStore
where
    A: Aggregate + Serialize + DeserializeOwned,
{
    async fn load(&self, stream: &StreamId) -> Result<Option<Snapshot<A, i64>>> {
        let storage_key = stream.to_storage_key();

        let row = sqlx::query("SELECT payload, version FROM snapshots WHERE stream_id = $1")
            .bind(&storage_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| EventSourcingError::Store {
                message: format!("Failed to load snapshot: {e}"),
            })?;

        match row {
            Some(row) => {
                let payload: serde_json::Value = row.get("payload");
                let version: i64 = row.get("version");
                let state: A =
                    serde_json::from_value(payload).map_err(|e| EventSourcingError::Deserialization {
                        source: e,
                    })?;
                Ok(Some(Snapshot { state, version }))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, stream: &StreamId, snapshot: &Snapshot<A, i64>) -> Result<()> {
        let storage_key = stream.to_storage_key();
        let payload =
            serde_json::to_value(&snapshot.state).map_err(|e| EventSourcingError::Serialization {
                source: e,
            })?;

        sqlx::query(
            "INSERT INTO snapshots (stream_id, version, payload, created_at)
             VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
             ON CONFLICT (stream_id)
             DO UPDATE SET version = $2, payload = $3, created_at = CURRENT_TIMESTAMP",
        )
        .bind(&storage_key)
        .bind(snapshot.version)
        .bind(&payload)
        .execute(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to save snapshot: {e}"),
        })?;

        Ok(())
    }
}
