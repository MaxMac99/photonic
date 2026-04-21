use std::sync::{Arc, RwLock};

use domain::medium::events::{
    TempCleanupCompletedEvent, TempCleanupFailedEvent, TempCleanupStartedEvent,
};
use event_sourcing::{
    bus::projection::ProjectionEventBus,
    stream::{definition::StreamExtract, linking_projection::StreamLinkingProjection},
};
use snafu::{ResultExt, Whatever};
use sqlx::{PgPool, Postgres, Transaction};

use super::stream_definitions::{medium_stream, metadata_stream, task_stream, user_stream};
use crate::{
    persistence::postgres::{
        checkpoint_store::PostgresTxCheckpointStore,
        events::{es_event_store::PostgresGlobalEventStore, type_registry::EventTypeRegistry},
        stream_link_store::PostgresStreamLinkStore,
        transaction_provider::PostgresTransactionProvider,
    },
    projections::{
        MediumProjection, MetadataProjection, RegisterProjection, TaskProjection, UserProjection,
    },
};

pub type PgProjectionBus = ProjectionEventBus<i64, Transaction<'static, Postgres>>;

/// Create the ProjectionEventBus with all projections registered.
/// The event type registry is auto-populated during projection registration.
pub fn build_projection_bus(
    db_pool: &PgPool,
) -> Result<(Arc<PgProjectionBus>, Arc<RwLock<EventTypeRegistry>>), Whatever> {
    let registry = Arc::new(RwLock::new(EventTypeRegistry::new()));

    let global_event_store = PostgresGlobalEventStore::new(db_pool.clone(), registry.clone());
    let checkpoint_store = PostgresTxCheckpointStore::new();
    let tx_provider = PostgresTransactionProvider::new(db_pool.clone());

    let bus = Arc::new(ProjectionEventBus::new(
        global_event_store,
        checkpoint_store,
        tx_provider,
    ));

    // Business projections — each call auto-registers event types in the registry
    {
        let mut reg = registry.write().unwrap();
        MediumProjection::register(&bus, &mut reg)
            .whatever_context("Failed to register MediumProjection")?;
        UserProjection::register(&bus, &mut reg)
            .whatever_context("Failed to register UserProjection")?;
        MetadataProjection::register(&bus, &mut reg)
            .whatever_context("Failed to register MetadataProjection")?;
        TaskProjection::register(&bus, &mut reg)
            .whatever_context("Failed to register TaskProjection")?;

        // TempCleanup events — persisted but no projections (only listeners)
        reg.register::<TempCleanupStartedEvent>();
        reg.register::<TempCleanupCompletedEvent>();
        reg.register::<TempCleanupFailedEvent>();
    }

    // Stream linking projection — populates event_streams table
    let extractors: Vec<Arc<dyn StreamExtract>> = vec![
        Arc::new(medium_stream()),
        Arc::new(user_stream()),
        Arc::new(task_stream()),
        Arc::new(metadata_stream()),
    ];
    bus.register_catch_all(StreamLinkingProjection::new(
        extractors,
        Arc::new(PostgresStreamLinkStore::new()),
    ))
    .whatever_context("Failed to register StreamLinkingProjection")?;

    Ok((bus, registry))
}
