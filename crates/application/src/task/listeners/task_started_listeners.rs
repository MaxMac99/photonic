use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::{metadata::events::MetadataEvent, task::TaskType};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    medium::events::TempCleanupStartedEvent,
    task::commands::{StartTaskCommand, StartTaskHandler},
};

#[derive(new)]
pub struct TaskStartedListeners {
    start_task_handler: Arc<StartTaskHandler>,
}

#[async_trait]
impl EventProcessor<MetadataEvent> for TaskStartedListeners {
    #[instrument(
        name = "TaskStartedListeners::MetadataEvent",
        skip(self, event),
    )]
    async fn process(&self, event: MetadataEvent) -> ApplicationResult<()> {
        let MetadataEvent::ExtractionStarted(event) = event else { return Ok(()) };
        info!(
            "Starting metadata extraction task for medium_id={} (leading_item_id={})",
            event.medium_id, event.leading_item_id,
        );

        self.start_task_handler
            .handle(StartTaskCommand {
                reference_id: event.leading_item_id,
                user_id: event.owner_id,
                task_type: TaskType::MetadataExtraction,
            })
            .await?;

        debug!(
            "Started MetadataExtraction task for medium_id={}",
            event.medium_id
        );

        Ok(())
    }
}

#[async_trait]
impl EventProcessor<TempCleanupStartedEvent> for TaskStartedListeners {
    #[instrument(
        name = "TaskStartedListeners::TempCleanupStartedEvent",
        skip(self, event),
        fields(
            event = "TempCleanupStartedEvent",
            sweep_id = %event.sweep_id,
        )
    )]
    async fn process(&self, event: TempCleanupStartedEvent) -> ApplicationResult<()> {
        info!(sweep_id = %event.sweep_id, "Creating and starting temp cleanup task");

        self.start_task_handler
            .handle(StartTaskCommand {
                reference_id: event.sweep_id,
                user_id: Uuid::nil(),
                task_type: TaskType::TempCleanup,
            })
            .await?;

        debug!(sweep_id = %event.sweep_id, "Started temp cleanup task");

        Ok(())
    }
}
