use std::marker::PhantomData;

use application::{error::ApplicationResult, event_store::EventStore};
use async_trait::async_trait;
use domain::{
    aggregate::{AggregateRoot, AggregateVersion},
    event::DomainEvent,
};
use sqlx::{PgPool, Row};

use super::repo_error;
use crate::events::StorableEvent;

pub struct PostgresEventStore<A: AggregateRoot> {
    pool: PgPool,
    _phantom: PhantomData<A>,
}

impl<A: AggregateRoot> PostgresEventStore<A> {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<A> EventStore<A> for PostgresEventStore<A>
where
    A: AggregateRoot + 'static,
    A::Event: StorableEvent,
{
    async fn load_events(
        &self,
        aggregate_id: &str,
        since_version: Option<AggregateVersion>,
    ) -> ApplicationResult<Vec<A::Event>> {
        let stream_id = format!("{}-{}", A::aggregate_type(), aggregate_id);
        let since = since_version.unwrap_or(0);

        let rows = sqlx::query(
            "SELECT payload FROM events WHERE stream_id = $1 AND version > $2 ORDER BY version",
        )
        .bind(&stream_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| application::error::ApplicationError::Domain {
            source: repo_error(e),
        })?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload: serde_json::Value = row.get("payload");
            let event: A::Event =
                serde_json::from_value(payload).map_err(|e| {
                    application::error::ApplicationError::Internal {
                        message: format!("Failed to deserialize event: {}", e),
                    }
                })?;
            events.push(event);
        }

        Ok(events)
    }

    async fn append_events(
        &self,
        aggregate_id: &str,
        expected_version: AggregateVersion,
        events: Vec<A::Event>,
    ) -> ApplicationResult<()> {
        let stream_id = format!("{}-{}", A::aggregate_type(), aggregate_id);
        let mut tx = self.pool.begin().await.map_err(|e| {
            application::error::ApplicationError::Domain {
                source: repo_error(e),
            }
        })?;

        for (i, event) in events.iter().enumerate() {
            let version = expected_version + i as i64 + 1;
            let event_type = event.event_type_name();
            let payload = serde_json::to_value(event).map_err(|e| {
                application::error::ApplicationError::Internal {
                    message: format!("Failed to serialize event: {}", e),
                }
            })?;
            let metadata = event.metadata();

            sqlx::query(
                "INSERT INTO events (stream_id, version, event_type, payload, event_id, occurred_at) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&stream_id)
            .bind(version)
            .bind(event_type)
            .bind(&payload)
            .bind(metadata.event_id)
            .bind(metadata.occurred_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| match &e {
                sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                    application::error::ApplicationError::Conflict {
                        message: format!(
                            "Concurrency conflict on stream '{}' at version {}",
                            stream_id, expected_version
                        ),
                    }
                }
                _ => application::error::ApplicationError::Domain {
                    source: repo_error(e),
                },
            })?;
        }

        tx.commit().await.map_err(|e| {
            application::error::ApplicationError::Domain {
                source: repo_error(e),
            }
        })?;

        Ok(())
    }
}
