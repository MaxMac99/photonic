use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::{
    medium::events::TempCleanupFailedEvent, metadata::events::MetadataExtractionFailedEvent,
    task::TaskType,
};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    task::commands::{FailTaskCommand, FailTaskHandler},
};

#[derive(new)]
pub struct TaskFailedListeners {
    fail_task_handler: Arc<FailTaskHandler>,
}

#[async_trait]
impl EventProcessor<MetadataExtractionFailedEvent> for TaskFailedListeners {
    type Error = crate::error::ApplicationError;

    #[instrument(
        name = "TaskFailedListeners::MetadataExtractionFailedEvent",
        skip(self, event)
    )]
    async fn process(&self, event: &MetadataExtractionFailedEvent) -> ApplicationResult<()> {
        info!(
            "Failed metadata extraction task for medium_id={} (leading_item_id={})",
            event.medium_id, event.leading_item_id,
        );

        self.fail_task_handler
            .handle(FailTaskCommand {
                reference_id: event.leading_item_id,
                user_id: event.owner_id,
                task_type: TaskType::MetadataExtraction,
                error: event.error.clone(),
            })
            .await?;

        debug!(
            "Failed MetadataExtraction task for medium_id={}",
            event.medium_id
        );

        Ok(())
    }
}

#[async_trait]
impl EventProcessor<TempCleanupFailedEvent> for TaskFailedListeners {
    type Error = crate::error::ApplicationError;

    #[instrument(
        name = "TaskFailedListeners::TempCleanupFailedEvent",
        skip(self, event),
        fields(
            event = "TempCleanupFailedEvent",
            sweep_id = %event.sweep_id,
        )
    )]
    async fn process(&self, event: &TempCleanupFailedEvent) -> ApplicationResult<()> {
        info!(
            sweep_id = %event.sweep_id,
            error = %event.error,
            "Failing temp cleanup task"
        );

        self.fail_task_handler
            .handle(FailTaskCommand {
                reference_id: event.sweep_id,
                user_id: Uuid::nil(),
                task_type: TaskType::TempCleanup,
                error: event.error.clone(),
            })
            .await?;

        debug!(sweep_id = %event.sweep_id, "Failed temp cleanup task");

        Ok(())
    }
}
