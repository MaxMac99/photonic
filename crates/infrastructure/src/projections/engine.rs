use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    time::Duration,
};

use futures_util::future::join_all;

use application::{
    error::{ApplicationError, ApplicationResult},
    projection::Projection,
};
use async_trait::async_trait;
use sqlx::{prelude::FromRow, PgPool};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

#[derive(Debug, FromRow)]
struct EventRow {
    pub global_sequence: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

// -- Internal type erasure (private to engine) --

type DecodedEvent = Box<dyn Any + Send + Sync>;

trait EventDecoder: Send + Sync {
    fn event_type_filter(&self) -> &'static [&'static str];
    fn decode(&self, payload: &serde_json::Value) -> ApplicationResult<DecodedEvent>;
}

#[async_trait]
trait ProjectionHandler: Send + Sync {
    fn name(&self) -> Cow<'static, str>;
    async fn handle_decoded(
        &self,
        event: &DecodedEvent,
        global_sequence: i64,
    ) -> ApplicationResult<()>;
}

struct TypedDecoder<E: StorableEvent> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E: StorableEvent + 'static> EventDecoder for TypedDecoder<E> {
    fn event_type_filter(&self) -> &'static [&'static str] {
        E::all_event_types()
    }

    fn decode(&self, payload: &serde_json::Value) -> ApplicationResult<DecodedEvent> {
        let event: E =
            serde_json::from_value(payload.clone()).map_err(|e| ApplicationError::Internal {
                message: format!("Failed to decode event: {}", e),
            })?;
        Ok(Box::new(event))
    }
}

struct TypedHandler<E: StorableEvent, P: Projection<E>> {
    projection: P,
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E: StorableEvent + 'static, P: Projection<E>> ProjectionHandler for TypedHandler<E, P> {
    fn name(&self) -> Cow<'static, str> {
        self.projection.name()
    }

    async fn handle_decoded(
        &self,
        event: &DecodedEvent,
        global_sequence: i64,
    ) -> ApplicationResult<()> {
        let event = event
            .downcast_ref::<E>()
            .expect("type mismatch in projection pipeline — this is a bug");
        self.projection.handle(event, global_sequence).await
    }
}

struct ProjectionGroup {
    decoder: Box<dyn EventDecoder>,
    handlers: Vec<Box<dyn ProjectionHandler>>,
}

// -- Public engine --

/// Background worker that polls the event store and dispatches events to projections.
///
/// Projections are grouped by event type. Events are decoded once per group,
/// then dispatched to all handlers in that group.
pub struct ProjectionEngine {
    pool: PgPool,
    groups: HashMap<TypeId, ProjectionGroup>,
    poll_interval: Duration,
    batch_size: i64,
}

impl ProjectionEngine {
    pub fn new(pool: PgPool, poll_interval: Duration, batch_size: i64) -> Self {
        Self {
            pool,
            groups: HashMap::new(),
            poll_interval,
            batch_size,
        }
    }

    /// Register a typed projection. Multiple projections for the same event type
    /// share a single decoder — the event is decoded once and passed to all handlers.
    pub fn register<E, P>(&mut self, projection: P)
    where
        E: StorableEvent + 'static,
        P: Projection<E> + 'static,
    {
        let type_id = TypeId::of::<E>();
        let group = self
            .groups
            .entry(type_id)
            .or_insert_with(|| ProjectionGroup {
                decoder: Box::new(TypedDecoder::<E> {
                    _phantom: std::marker::PhantomData,
                }),
                handlers: Vec::new(),
            });
        group.handlers.push(Box::new(TypedHandler::<E, P> {
            projection,
            _phantom: std::marker::PhantomData,
        }));
    }

    /// Run the projection engine until cancellation.
    pub async fn run(&self, shutdown: CancellationToken) {
        let handler_count: usize = self.groups.values().map(|g| g.handlers.len()).sum();
        info!(
            groups = self.groups.len(),
            handlers = handler_count,
            poll_interval_ms = self.poll_interval.as_millis() as u64,
            batch_size = self.batch_size,
            "Projection engine started"
        );

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    info!("Projection engine shutting down");
                    break;
                }
                _ = tokio::time::sleep(self.poll_interval) => {
                    self.process_all().await;
                }
            }
        }
    }

    async fn process_all(&self) {
        for group in self.groups.values() {
            for handler in &group.handlers {
                if let Err(e) = self.process_handler(group, handler.as_ref()).await {
                    error!(
                        projection = %handler.name(),
                        error = %e,
                        "Projection processing failed, will retry next poll"
                    );
                }
            }
        }
    }

    async fn process_handler(
        &self,
        group: &ProjectionGroup,
        handler: &dyn ProjectionHandler,
    ) -> ApplicationResult<()> {
        let name = handler.name();
        let checkpoint = self.load_checkpoint(&name).await?;
        let filter = group.decoder.event_type_filter();
        let events = self.fetch_events(checkpoint, filter).await?;

        if events.is_empty() {
            return Ok(());
        }

        debug!(
            projection = %name,
            events_count = events.len(),
            from_sequence = checkpoint,
            "Processing events batch"
        );

        for event_row in events {
            let decoded = group.decoder.decode(&event_row.payload)?;

            if let Err(e) = handler
                .handle_decoded(&decoded, event_row.global_sequence)
                .await
            {
                warn!(
                    projection = %name,
                    event_type = event_row.event_type,
                    global_sequence = event_row.global_sequence,
                    error = %e,
                    "Projection failed to handle event, stopping at this checkpoint"
                );
                return Err(e);
            }

            self.save_checkpoint(&name, event_row.global_sequence)
                .await?;
        }

        Ok(())
    }

    async fn load_checkpoint(&self, projection_name: &str) -> ApplicationResult<i64> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_global_sequence FROM projection_checkpoints WHERE projection_name = $1",
        )
        .bind(projection_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal {
            message: format!("Failed to load checkpoint: {}", e),
        })?;

        Ok(row.map(|r| r.0).unwrap_or(0))
    }

    async fn save_checkpoint(
        &self,
        projection_name: &str,
        global_sequence: i64,
    ) -> ApplicationResult<()> {
        sqlx::query(
            "INSERT INTO projection_checkpoints (projection_name, last_global_sequence, updated_at)
             VALUES ($1, $2, CURRENT_TIMESTAMP)
             ON CONFLICT (projection_name)
             DO UPDATE SET last_global_sequence = $2, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(projection_name)
        .bind(global_sequence)
        .execute(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal {
            message: format!("Failed to save checkpoint: {}", e),
        })?;

        Ok(())
    }

    async fn fetch_events(
        &self,
        after_sequence: i64,
        event_types: &[&str],
    ) -> ApplicationResult<Vec<EventRow>> {
        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT global_sequence, event_type, payload
             FROM events
             WHERE global_sequence > $1 AND event_type = ANY($2)
             ORDER BY global_sequence
             LIMIT $3",
        )
        .bind(after_sequence)
        .bind(event_types)
        .bind(self.batch_size)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApplicationError::Internal {
            message: format!("Failed to fetch events: {}", e),
        })?;

        Ok(rows)
    }
}
