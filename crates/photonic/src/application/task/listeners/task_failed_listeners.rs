use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::domain::task::TaskType;
use crate::{
    application::{
        error::ApplicationResult,
        event_bus::EventProcessor,
        task::commands::{FailTaskCommand, FailTaskHandler},
    },
    domain::{
        medium::events::TempCleanupFailedEvent, metadata::events::MetadataExtractionFailedEvent,
    },
};

#[derive(new)]
pub struct TaskFailedListeners {
    fail_task_handler: Arc<FailTaskHandler>,
}

#[async_trait]
impl EventProcessor<MetadataExtractionFailedEvent> for TaskFailedListeners {
    #[instrument(
        name = "TaskFailedListeners::MetadataExtractionFailedEvent",
        skip(self, event),
        fields(
            event = "MetadataExtractionFailedEvent",
            medium_id = %event.medium_id,
            medium_item_id = %event.leading_item_id,
            user_id = %event.owner_id,
        )
    )]
    async fn process(&self, event: MetadataExtractionFailedEvent) -> ApplicationResult<()> {
        info!(
            "Failed metadata extraction task for medium_id={} (leading_item_id={})",
            event.medium_id, event.leading_item_id,
        );

        self.fail_task_handler
            .handle(FailTaskCommand {
                reference_id: event.leading_item_id,
                user_id: event.owner_id,
                task_type: TaskType::MetadataExtraction,
                error: event.error,
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
    #[instrument(
        name = "TaskFailedListeners::TempCleanupFailedEvent",
        skip(self, event),
        fields(
            event = "TempCleanupFailedEvent",
            sweep_id = %event.sweep_id,
        )
    )]
    async fn process(&self, event: TempCleanupFailedEvent) -> ApplicationResult<()> {
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
                error: event.error,
            })
            .await?;

        debug!(sweep_id = %event.sweep_id, "Failed temp cleanup task");

        Ok(())
    }
}
