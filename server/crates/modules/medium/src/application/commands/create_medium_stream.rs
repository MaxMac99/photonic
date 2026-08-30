use std::{path::PathBuf, sync::Arc};

use byte_unit::Byte;
use chrono::{DateTime, FixedOffset};
use derive_new::new;
use kernel::{
    app_error::{ApplicationError, ApplicationResult},
    error::format_domain_error_with_backtrace as format_domain_error,
    event_bus::PublishEvent,
    FileLocation, Filename, MediumId, Priority, StorageTier, UserId,
};
use mime::Mime;
use tokio::io::AsyncRead;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    application::ports::{FileStorage, QuotaPort},
    domain::{
        events::MediumCreatedEvent, Medium, MediumCreateRequest, MediumItemCreateRequest,
        MediumItemType, MediumType,
    },
};

pub struct CreateMediumStreamCommand {
    pub user_id: UserId,
    pub stream: Box<dyn AsyncRead + Send + Unpin>,
    pub file_size: Byte,
    pub mime_type: Mime,
    pub filename: String,
    pub medium_type: Option<MediumType>,
    pub priority: Option<i32>,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

#[derive(new)]
pub struct CreateMediumStreamHandler {
    file_storage: Arc<dyn FileStorage>,
    quota: Arc<dyn QuotaPort>,
    event_bus: Arc<dyn PublishEvent<MediumCreatedEvent>>,
}

impl CreateMediumStreamHandler {
    #[instrument(skip(self, command), fields(
        user_id = %command.user_id,
        file_size = %command.file_size.as_u64(),
        mime_type = %command.mime_type,
        filename = %command.filename
    ))]
    pub async fn handle(&self, command: CreateMediumStreamCommand) -> ApplicationResult<MediumId> {
        info!("Creating medium from stream");

        let reservation = self
            .quota
            .reserve(command.user_id, command.file_size)
            .await?;

        match self.create(command).await {
            Ok(medium_id) => {
                // The medium exists, so the reserved bytes are consumed.
                // A failed commit only skips the finality marker — the
                // bytes are already accounted for, so just warn.
                if let Err(e) = self.quota.commit(reservation).await {
                    warn!(
                        medium_id = %medium_id,
                        error = %e,
                        "Failed to commit quota reservation"
                    );
                }
                info!(medium_id = %medium_id, "Medium created successfully from stream");
                Ok(medium_id)
            }
            Err(e) => {
                if let Err(release_err) = self.quota.release(reservation).await {
                    error!(
                        error = %release_err,
                        "CRITICAL: Failed to release quota reservation. \
                         Manual intervention required"
                    );
                }
                Err(e)
            }
        }
    }

    async fn create(&self, command: CreateMediumStreamCommand) -> ApplicationResult<MediumId> {
        let medium_type = command
            .medium_type
            .unwrap_or_else(|| MediumType::from(command.mime_type.clone()));
        let filename =
            Filename::new(&command.filename).map_err(|e| ApplicationError::Domain { source: e })?;
        let priority = command.priority.map(Priority::new).unwrap_or_default();

        let temp_file_id = Uuid::new_v4();
        let temp_location = FileLocation::new(
            StorageTier::Temporary,
            PathBuf::from(format!("{}.{}", temp_file_id, filename.extension())),
        );

        let medium_item_request = MediumItemCreateRequest {
            owner_id: command.user_id,
            medium_item_type: MediumItemType::Original,
            mime: command.mime_type,
            filename,
            filesize: command.file_size,
            priority,
            dimensions: None,
            locations: vec![temp_location.clone()],
        };

        let medium_request = MediumCreateRequest {
            owner_id: command.user_id,
            medium_type,
            taken_at: command.date_taken,
            camera_make: command.camera_make,
            camera_model: command.camera_model,
            medium_item: medium_item_request,
        };
        let (medium, created_event) = Medium::new(medium_request)?;
        let medium_id = medium.id;

        debug!(
            medium_id = %medium_id,
            temp_location = ?temp_location.relative_path,
            "Storing file and persisting events"
        );

        // Store file to temporary storage
        self.file_storage
            .store_file_stream(&temp_location, command.stream)
            .await
            .map_err(|e| {
                error!(
                    medium_id = %medium_id,
                    error = %format_domain_error(&e),
                    "File storage failed"
                );
                ApplicationError::Domain { source: e }
            })?;

        // Publish event — persists to event store, then dispatches to listeners
        self.event_bus.publish(created_event).await.map_err(|e| {
            error!(
                medium_id = %medium_id,
                error = %e,
                "Failed to publish event"
            );
            e
        })?;

        Ok(medium_id)
    }
}
