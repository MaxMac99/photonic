use crate::events::{EventAppender, TransactionProvider, TransactionalEventAppender};
use crate::persistence::postgres::events::storable_event::StorableEvent;
use application::error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::debug;

/// Persists a single event to the events table.
pub struct PostgresEventAppender {
    pool: PgPool,
}

impl PostgresEventAppender {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<E: StorableEvent + 'static> TransactionalEventAppender<E, Transaction<'static, Postgres>>
    for PostgresEventAppender
{
    async fn append(
        &self,
        event: &E,
        tx: &mut Transaction<'static, Postgres>,
    ) -> ApplicationResult<()> {
        let stream_id = format!("{}-{}", E::aggregate_type(), event.aggregate_id());
        let event_type = event.event_type_name();
        let metadata = event.metadata();
        let version = metadata.expected_version + 1;

        let payload = serde_json::to_value(event).map_err(|e| ApplicationError::Internal {
            message: format!("Failed to serialize event: {}", e),
        })?;

        sqlx::query(
            "INSERT INTO events (stream_id, version, event_type, payload, event_id, occurred_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&stream_id)
        .bind(version)
        .bind(event_type)
        .bind(&payload)
        .bind(metadata.event_id)
        .bind(metadata.occurred_at)
        .execute(tx)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                ApplicationError::Conflict {
                    message: format!(
                        "Concurrency conflict on stream '{}' at version {}",
                        stream_id, metadata.expected_version
                    ),
                }
            }
            _ => ApplicationError::Internal {
                message: format!("Failed to persist event: {}", e),
            },
        })?;

        debug!(
            stream_id = stream_id,
            version = version,
            event_type = event_type,
            "Event persisted"
        );

        Ok(())
    }
}
