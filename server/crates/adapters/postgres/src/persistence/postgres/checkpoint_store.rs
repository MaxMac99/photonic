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

/// Coordination-safe transactional checkpoint store (ADR 0006) for
/// multi-instance deployments. Uses the same tables as
/// [`PostgresTxCheckpointStore`] but claims the consumer with a
/// transaction-scoped advisory lock before touching the checkpoint row:
///
/// - Concurrent instances serialize their checkpoint transactions per
///   consumer (the lock blocks until the other instance commits/rolls back).
/// - The checkpoint only ever moves forward.
/// - A save for an event another instance already committed fails, so the
///   duplicate projection transaction rolls back instead of double-applying.
///
/// The cost is some latency under contention — the trade-off ADR 0006
/// makes instead of an external broker.
pub struct PostgresCoordinatedTxCheckpointStore;

impl PostgresCoordinatedTxCheckpointStore {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TxCheckpointStore<i64, Transaction<'static, Postgres>>
    for PostgresCoordinatedTxCheckpointStore
{
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
        // Claim this consumer for the transaction. Blocks while another
        // instance holds it; the lock is released at commit/rollback.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
            .bind(consumer_name)
            .execute(&mut **tx)
            .await
            .map_err(|e| EventSourcingError::Store {
                message: format!("Failed to claim checkpoint lock: {e}"),
            })?;

        let stored: Option<(i64,)> = sqlx::query_as(
            "SELECT last_global_sequence FROM projection_checkpoints WHERE projection_name = $1",
        )
        .bind(consumer_name)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to load checkpoint: {e}"),
        })?;

        match stored {
            // Fresh consumer: insert the first checkpoint.
            None => {
                sqlx::query(
                    "INSERT INTO projection_checkpoints (projection_name, last_global_sequence, updated_at)
                     VALUES ($1, $2, CURRENT_TIMESTAMP)",
                )
                .bind(consumer_name)
                .bind(sequence)
                .execute(&mut **tx)
                .await
                .map_err(|e| EventSourcingError::Store {
                    message: format!("Failed to save checkpoint: {e}"),
                })?;
            }
            // Normal path: advance the checkpoint for this consumer.
            Some((stored,)) if stored < sequence => {
                sqlx::query(
                    "UPDATE projection_checkpoints
                     SET last_global_sequence = $2, updated_at = CURRENT_TIMESTAMP
                     WHERE projection_name = $1",
                )
                .bind(consumer_name)
                .bind(sequence)
                .execute(&mut **tx)
                .await
                .map_err(|e| EventSourcingError::Store {
                    message: format!("Failed to save checkpoint: {e}"),
                })?;
            }
            // Another instance already committed this exact event. Fail so
            // the caller rolls back its duplicate projection work.
            Some((stored,)) if stored == sequence => {
                return Err(EventSourcingError::Store {
                    message: format!(
                        "checkpoint for '{consumer_name}' already at {stored}; \
                         event already processed by another instance"
                    ),
                });
            }
            // A later event already advanced the checkpoint (out-of-order
            // commit across instances). Keep this transaction's projection
            // work, but do not regress the checkpoint.
            Some(_) => {}
        }

        Ok(())
    }
}
