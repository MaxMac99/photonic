use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use kernel::{app_error::ApplicationResult, event_bus::EventProcessor};
use medium::domain::events::TempCleanupCompletedEvent;
use metadata::domain::events::MetadataExtractedEvent;
use task::{
    application::commands::{CompleteTaskCommand, CompleteTaskHandler},
    domain::TaskType,
};
use tracing::{debug, info, instrument};
use uuid::Uuid;

#[derive(new)]
pub struct TaskCompletedListeners {
    complete_task_handler: Arc<CompleteTaskHandler>,
}

#[async_trait]
impl EventProcessor<MetadataExtractedEvent> for TaskCompletedListeners {
    type Error = kernel::app_error::ApplicationError;

    #[instrument(
        name = "TaskCompletedListeners::MetadataExtractedEvent",
        skip(self, event)
    )]
    async fn process(&self, event: &MetadataExtractedEvent) -> ApplicationResult<()> {
        info!(
            "Completing metadata extraction task for medium_id={} (leading_item_id={})",
            event.medium_id, event.leading_item_id,
        );

        self.complete_task_handler
            .handle(CompleteTaskCommand {
                reference_id: event.leading_item_id,
                user_id: event.owner_id,
                task_type: TaskType::MetadataExtraction,
            })
            .await?;

        debug!(
            "Completed MetadataExtraction task for medium_id={}",
            event.medium_id
        );

        Ok(())
    }
}

#[async_trait]
impl EventProcessor<TempCleanupCompletedEvent> for TaskCompletedListeners {
    type Error = kernel::app_error::ApplicationError;

    #[instrument(
        name = "TaskCompletedListeners::TempCleanupCompletedEvent",
        skip(self, event),
        fields(
            event = "TempCleanupCompletedEvent",
            sweep_id = %event.sweep_id,
            items_cleaned = event.items_cleaned,
        )
    )]
    async fn process(&self, event: &TempCleanupCompletedEvent) -> ApplicationResult<()> {
        info!(
            sweep_id = %event.sweep_id,
            items_cleaned = event.items_cleaned,
            "Completing temp cleanup task"
        );

        self.complete_task_handler
            .handle(CompleteTaskCommand {
                reference_id: event.sweep_id,
                user_id: Uuid::nil(),
                task_type: TaskType::TempCleanup,
            })
            .await?;

        debug!(sweep_id = %event.sweep_id, "Completed temp cleanup task");

        Ok(())
    }
}
