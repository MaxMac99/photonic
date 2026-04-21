use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use event_sourcing::{
    aggregate::event_store::AggregateEventStore,
    error::{EventSourcingError, Result},
    persistence::event_store::StoredEvent,
    stream::stream_id::StreamId,
};
use sqlx::{PgPool, Row};

use super::type_registry::EventTypeRegistry;

/// Postgres implementation of `AggregateEventStore<i64>`.
///
/// Loads aggregate streams by joining `event_streams` with `events`.
/// Read-only — writing is handled by the event bus + `StreamLinkingProjection`.
pub struct PostgresAggregateEventStore {
    pool: PgPool,
    registry: Arc<RwLock<EventTypeRegistry>>,
}

impl PostgresAggregateEventStore {
    pub fn new(pool: PgPool, registry: Arc<RwLock<EventTypeRegistry>>) -> Self {
        Self { pool, registry }
    }
}

#[async_trait]
impl AggregateEventStore<i64> for PostgresAggregateEventStore {
    async fn load_stream(
        &self,
        stream: &StreamId,
        after_version: i64,
    ) -> Result<Vec<StoredEvent<i64>>> {
        let rows = sqlx::query(
            "SELECT e.global_sequence, e.event_type, e.payload
             FROM event_streams es
             JOIN events e ON es.global_sequence = e.global_sequence
             WHERE es.stream_category = $1 AND es.stream_id = $2 AND es.stream_version > $3
             ORDER BY es.stream_version",
        )
        .bind(stream.aggregate_type().name())
        .bind(stream.id())
        .bind(after_version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to load stream: {e}"),
        })?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let global_sequence: i64 = row.get("global_sequence");
            let event_type: String = row.get("event_type");
            let payload: serde_json::Value = row.get("payload");

            let registry = self.registry.read().unwrap();
            let event = registry.deserialize(&event_type, &payload)?;
            events.push(StoredEvent {
                sequence: global_sequence,
                event,
            });
        }

        Ok(events)
    }
}
