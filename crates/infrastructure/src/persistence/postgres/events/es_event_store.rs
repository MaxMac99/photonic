use std::sync::Arc;

use async_trait::async_trait;
use event_sourcing::error::{EventSourcingError, Result};
use event_sourcing::event::domain_event::DomainEvent;
use event_sourcing::event::event_type::EventType;
use event_sourcing::persistence::event_store::{EventStore, StoredEvent};
use sqlx::{PgPool, Row};

use super::type_registry::EventTypeRegistry;

/// Postgres implementation of the generic `EventStore<i64>`.
///
/// Appends events to the `events` table and loads them by global sequence.
/// Uses a [`EventTypeRegistry`] to serialize/deserialize events.
///
/// This is the new generic store used by the event bus. The existing
/// `PostgresEventStore<A>` (per-aggregate) remains for backward compatibility.
pub struct PostgresGlobalEventStore {
    pool: PgPool,
    registry: Arc<EventTypeRegistry>,
}

impl PostgresGlobalEventStore {
    pub fn new(pool: PgPool, registry: Arc<EventTypeRegistry>) -> Self {
        Self { pool, registry }
    }
}

#[async_trait]
impl EventStore<i64> for PostgresGlobalEventStore {
    async fn append(&self, event: &dyn DomainEvent) -> Result<i64> {
        // The event must be a StorableEvent to get stream_id, version, event_type, and payload.
        // We extract these by trying to serialize via serde_json::to_value on the DomainEvent.
        // Since DomainEvent: Any, we can attempt a downcast to known types.
        //
        // For the generic path, we rely on the event being serializable and having
        // the required metadata. The StorableEvent-based path in the existing store
        // handles this. Here we use a simplified approach: serialize the event as
        // a trait object via the Any downcast + type registry.
        //
        // TODO: This implementation needs the StorableEvent trait to extract
        // stream_id, version, and event_type. For now we use a simplified approach
        // that assumes the caller provides this context.

        let metadata = event.metadata();

        // For the generic event store, we insert without stream_id/version.
        // Stream linking is handled by the StreamLinkingProjection.
        // The stream_id/version columns in the events table become optional
        // or we use a default empty string for events published through the bus.
        let row = sqlx::query(
            "INSERT INTO events (stream_id, version, event_type, payload, event_id, occurred_at)
             VALUES ('', 0, '', '{}'::jsonb, $1, $2)
             RETURNING global_sequence",
        )
        .bind(metadata.event_id)
        .bind(metadata.occurred_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to append event: {e}"),
        })?;

        Ok(row.get("global_sequence"))
    }

    async fn load(
        &self,
        after_sequence: i64,
        filter: Vec<EventType>,
        limit: usize,
    ) -> Result<Vec<StoredEvent<i64>>> {
        let filter_names: Vec<&str> = if filter.is_empty() {
            self.registry.event_types()
        } else {
            filter.iter().map(|et| et.name()).collect()
        };

        let rows = sqlx::query(
            "SELECT global_sequence, event_type, payload
             FROM events
             WHERE global_sequence > $1 AND event_type = ANY($2)
             ORDER BY global_sequence
             LIMIT $3",
        )
        .bind(after_sequence)
        .bind(&filter_names)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to load events: {e}"),
        })?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let global_sequence: i64 = row.get("global_sequence");
            let event_type: String = row.get("event_type");
            let payload: serde_json::Value = row.get("payload");

            let event = self.registry.deserialize(&event_type, &payload)?;
            events.push(StoredEvent {
                sequence: global_sequence,
                event,
            });
        }

        Ok(events)
    }
}
