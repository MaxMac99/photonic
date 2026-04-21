use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use event_sourcing::{
    error::{EventSourcingError, Result},
    event::{domain_event::DomainEvent, event_type::EventType},
    persistence::event_store::{EventStore, StoredEvent},
};
use sqlx::{PgPool, Row};

use super::type_registry::EventTypeRegistry;

/// Postgres implementation of the generic `EventStore<i64>`.
///
/// Appends events to the `events` table and loads them by global sequence.
/// Uses a shared [`EventTypeRegistry`] to serialize/deserialize events.
/// The registry is populated automatically during projection registration.
pub struct PostgresGlobalEventStore {
    pool: PgPool,
    registry: Arc<RwLock<EventTypeRegistry>>,
}

impl PostgresGlobalEventStore {
    pub fn new(pool: PgPool, registry: Arc<RwLock<EventTypeRegistry>>) -> Self {
        Self { pool, registry }
    }
}

#[async_trait]
impl EventStore<i64> for PostgresGlobalEventStore {
    async fn append(&self, event: &dyn DomainEvent) -> Result<i64> {
        // Serialize while holding the lock, then drop before await
        let (ser, metadata_event_id, metadata_occurred_at) = {
            let registry = self.registry.read().unwrap();
            let ser = registry
                .serialize(event)
                .ok_or_else(|| EventSourcingError::Store {
                    message: "Event type not registered in EventTypeRegistry. \
                         Every event published through the bus must be registered."
                        .into(),
                })??;
            let metadata = event.metadata();
            (ser, metadata.event_id, metadata.occurred_at)
        };

        let row = sqlx::query(
            "INSERT INTO events (event_type, payload, event_id, occurred_at)
             VALUES ($1, $2, $3, $4)
             RETURNING global_sequence",
        )
        .bind(&ser.event_type)
        .bind(&ser.payload)
        .bind(metadata_event_id)
        .bind(metadata_occurred_at)
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
        // Collect filter names while holding the lock, then drop before await
        let filter_names: Vec<String> = {
            let registry = self.registry.read().unwrap();
            if filter.is_empty() {
                registry
                    .event_types()
                    .into_iter()
                    .map(String::from)
                    .collect()
            } else {
                filter.iter().map(|et| et.name().to_string()).collect()
            }
        };

        let filter_refs: Vec<&str> = filter_names.iter().map(|s| s.as_str()).collect();

        let rows = sqlx::query(
            "SELECT global_sequence, event_type, payload
             FROM events
             WHERE global_sequence > $1 AND event_type = ANY($2)
             ORDER BY global_sequence
             LIMIT $3",
        )
        .bind(after_sequence)
        .bind(&filter_refs)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to load events: {e}"),
        })?;

        let registry = self.registry.read().unwrap();
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let global_sequence: i64 = row.get("global_sequence");
            let event_type: String = row.get("event_type");
            let payload: serde_json::Value = row.get("payload");

            let event = registry.deserialize(&event_type, &payload)?;
            events.push(StoredEvent {
                sequence: global_sequence,
                event,
            });
        }

        Ok(events)
    }
}
