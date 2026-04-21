use std::sync::Arc;

use chrono::{DateTime, Utc};
use derive_new::new;
use domain::medium::{
    events::{TempCleanupCompletedEvent, TempCleanupFailedEvent, TempCleanupStartedEvent},
    StorageTier,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    error::ApplicationResult,
    event_bus::PublishEvent,
    medium::ports::{FileStorage, MediumRepository},
};

pub trait PublishCleanupEvent:
    PublishEvent<TempCleanupStartedEvent>
    + PublishEvent<TempCleanupCompletedEvent>
    + PublishEvent<TempCleanupFailedEvent>
{
}

impl<T> PublishCleanupEvent for T where
    T: PublishEvent<TempCleanupStartedEvent>
        + PublishEvent<TempCleanupCompletedEvent>
        + PublishEvent<TempCleanupFailedEvent>
{
}

pub struct CleanupExpiredTempStorageCommand {
    pub cutoff: DateTime<Utc>,
}

#[derive(new)]
pub struct CleanupExpiredTempStorageHandler {
    medium_repository: Arc<dyn MediumRepository>,
    file_storage: Arc<dyn FileStorage>,
    event_bus: Arc<dyn PublishCleanupEvent>,
}

impl CleanupExpiredTempStorageHandler {
    pub async fn handle(&self, command: CleanupExpiredTempStorageCommand) -> ApplicationResult<()> {
        let sweep_id = Uuid::new_v4();

        info!(sweep_id = %sweep_id, cutoff = %command.cutoff, "Starting temp storage cleanup sweep");

        self.event_bus
            .publish(TempCleanupStartedEvent::new(sweep_id))
            .await?;

        match self.execute_cleanup(sweep_id, command.cutoff).await {
            Ok(items_cleaned) => {
                info!(sweep_id = %sweep_id, items_cleaned, "Temp storage cleanup sweep completed");
                self.event_bus
                    .publish(TempCleanupCompletedEvent::new(sweep_id, items_cleaned))
                    .await?;
                Ok(())
            }
            Err(e) => {
                error!(sweep_id = %sweep_id, error = %e, "Temp storage cleanup sweep failed");
                self.event_bus
                    .publish(TempCleanupFailedEvent::new(sweep_id, e.to_string()))
                    .await?;
                Err(e)
            }
        }
    }

    async fn execute_cleanup(
        &self,
        sweep_id: Uuid,
        cutoff: DateTime<Utc>,
    ) -> ApplicationResult<usize> {
        let expired = self
            .medium_repository
            .find_expired_temp_locations(cutoff)
            .await?;

        if expired.is_empty() {
            info!(sweep_id = %sweep_id, "No expired temp locations found");
            return Ok(0);
        }

        info!(sweep_id = %sweep_id, count = expired.len(), "Found expired temp locations");

        let mut cleaned = 0;

        for location in &expired {
            match self.file_storage.delete_file(&location.temp_location).await {
                Ok(()) => {
                    let mut medium = self
                        .medium_repository
                        .find_by_id(location.medium_id, location.owner_id)
                        .await?;

                    if let Some(medium) = medium.as_mut() {
                        if let Some(item) = medium.find_item_mut(location.item_id) {
                            item.remove_location(StorageTier::Temporary);
                            self.medium_repository.save(medium).await?;
                            cleaned += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        sweep_id = %sweep_id,
                        medium_id = %location.medium_id,
                        item_id = %location.item_id,
                        error = %e,
                        "Failed to delete temp file, skipping"
                    );
                }
            }
        }

        Ok(cleaned)
    }
}
