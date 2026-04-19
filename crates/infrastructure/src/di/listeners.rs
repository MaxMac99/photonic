use application::{
    medium::listeners::{MediumMetadataEnrichmentListener, MoveToPermanentStorageListener},
    metadata::listeners::MetadataExtractionListeners,
    task::listeners::{
        TaskCompletedListeners, TaskCreationListeners, TaskFailedListeners, TaskStartedListeners,
    },
};
use domain::{
    medium::events::{
        MediumCreatedEvent, MediumUpdatedEvent, TempCleanupCompletedEvent, TempCleanupFailedEvent,
        TempCleanupStartedEvent,
    },
    metadata::events::{
        MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
    },
};

use super::factories::ApplicationHandlers;
use crate::{
    persistence::postgres::events::type_registry::EventTypeRegistry,
    projections::{register_listener, PgProjectionBus},
};

/// Register all side-effect listeners on the projection bus.
///
/// Listeners are registered as checkpointed projection handlers. They are
/// replayed from their checkpoint during `bus.start()` and receive live
/// events after. Must be called BEFORE `bus.start()`.
pub fn register_listeners(
    handlers: &ApplicationHandlers,
    bus: &PgProjectionBus,
    registry: &mut EventTypeRegistry,
) -> event_sourcing::error::Result<()> {
    // -- Medium event listeners --

    register_listener::<MediumCreatedEvent, _>(
        bus,
        registry,
        TaskCreationListeners::new(handlers.processing.create_task.clone()),
    )?;

    register_listener::<MediumCreatedEvent, _>(
        bus,
        registry,
        MetadataExtractionListeners::new(handlers.metadata.extract_metadata_handler.clone()),
    )?;

    register_listener::<MediumUpdatedEvent, _>(
        bus,
        registry,
        MoveToPermanentStorageListener::new(handlers.medium.move_to_permanent_storage.clone()),
    )?;

    // -- Metadata event listeners --

    register_listener::<MetadataExtractionStartedEvent, _>(
        bus,
        registry,
        TaskStartedListeners::new(handlers.processing.start_task.clone()),
    )?;

    register_listener::<MetadataExtractedEvent, _>(
        bus,
        registry,
        TaskCompletedListeners::new(handlers.processing.complete_task.clone()),
    )?;

    register_listener::<MetadataExtractionFailedEvent, _>(
        bus,
        registry,
        TaskFailedListeners::new(handlers.processing.fail_task.clone()),
    )?;

    register_listener::<MetadataExtractedEvent, _>(
        bus,
        registry,
        MediumMetadataEnrichmentListener::new(handlers.medium.enrich_medium_with_metadata.clone()),
    )?;

    // -- TempCleanup event listeners --

    register_listener::<TempCleanupStartedEvent, _>(
        bus,
        registry,
        TaskStartedListeners::new(handlers.processing.start_task.clone()),
    )?;

    register_listener::<TempCleanupCompletedEvent, _>(
        bus,
        registry,
        TaskCompletedListeners::new(handlers.processing.complete_task.clone()),
    )?;

    register_listener::<TempCleanupFailedEvent, _>(
        bus,
        registry,
        TaskFailedListeners::new(handlers.processing.fail_task.clone()),
    )?;

    Ok(())
}
