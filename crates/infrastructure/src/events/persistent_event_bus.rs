use std::sync::Arc;

use application::{
    error::{ApplicationError, ApplicationResult},
    event_bus::PublishEvent,
};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{debug, error};

use super::{storable_event::StorableEvent, EventBus};

/// Event bus that persists events to the event store before dispatching
/// to in-memory listeners. Wraps the in-memory `EventBus`.
///
/// Flow: persist to DB → dispatch to listeners
pub struct PersistentEventBus {
    pool: PgPool,
    inner: Arc<EventBus>,
}

impl PersistentEventBus {
    pub fn new(pool: PgPool, inner: Arc<EventBus>) -> Self {
        Self { pool, inner }
    }
}

#[async_trait]
impl<E: StorableEvent + 'static> PublishEvent<E> for PersistentEventBus {
    async fn publish(&self, event: E) -> ApplicationResult<()> {
        let stream_id = format!("{}-{}", E::aggregate_type(), event.aggregate_id());
        let event_type = event.event_type_name();
        let metadata = event.metadata().clone();
        let version = metadata.expected_version + 1;

        // Serialize
        let payload = serde_json::to_value(&event).map_err(|e| ApplicationError::Internal {
            message: format!("Failed to serialize event: {}", e),
        })?;

        // Persist with optimistic concurrency — unique(stream_id, version) catches conflicts
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
        .execute(&self.pool)
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

        // Dispatch to in-memory listeners
        if let Err(e) = self.inner.publish(event).await {
            error!(
                stream_id = stream_id,
                error = %e,
                "Event persisted but failed to dispatch to listeners"
            );
        }

        Ok(())
    }
}
