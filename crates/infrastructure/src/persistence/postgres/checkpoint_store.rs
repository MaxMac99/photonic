use async_trait::async_trait;
use event_sourcing::{
    error::{EventSourcingError, Result},
    persistence::checkpoint_store::{CheckpointStore, TxCheckpointStore},
};
use sqlx::{PgPool, Postgres, Transaction};

/// Non-transactional checkpoint store using a connection pool.
pub struct PostgresCheckpointStore {
    pool: PgPool,
}

impl PostgresCheckpointStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CheckpointStore<i64> for PostgresCheckpointStore {
    async fn load(&self, consumer_name: &str) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_global_sequence FROM projection_checkpoints WHERE projection_name = $1",
        )
        .bind(consumer_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to load checkpoint: {e}"),
        })?;

        Ok(row.map(|r| r.0).unwrap_or(0))
    }

    async fn save(&self, consumer_name: &str, sequence: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO projection_checkpoints (projection_name, last_global_sequence, updated_at)
             VALUES ($1, $2, CURRENT_TIMESTAMP)
             ON CONFLICT (projection_name)
             DO UPDATE SET last_global_sequence = $2, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(consumer_name)
        .bind(sequence)
        .execute(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to save checkpoint: {e}"),
        })?;

        Ok(())
    }
}

/// Transactional checkpoint store that operates within an existing transaction.
pub struct PostgresTxCheckpointStore;

impl PostgresTxCheckpointStore {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TxCheckpointStore<i64, Transaction<'static, Postgres>> for PostgresTxCheckpointStore {
    async fn load(
        &self,
        consumer_name: &str,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_global_sequence FROM projection_checkpoints WHERE projection_name = $1",
        )
        .bind(consumer_name)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to load checkpoint: {e}"),
        })?;

        Ok(row.map(|r| r.0).unwrap_or(0))
    }

    async fn save(
        &self,
        consumer_name: &str,
        sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO projection_checkpoints (projection_name, last_global_sequence, updated_at)
             VALUES ($1, $2, CURRENT_TIMESTAMP)
             ON CONFLICT (projection_name)
             DO UPDATE SET last_global_sequence = $2, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(consumer_name)
        .bind(sequence)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to save checkpoint: {e}"),
        })?;

        Ok(())
    }
}
