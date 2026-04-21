use std::{path::PathBuf, sync::Arc};

use byte_unit::Byte;
use chrono::{DateTime, FixedOffset};
use derive_new::new;
use domain::{
    error::format_error_with_backtrace as format_domain_error,
    medium::{
        events::MediumCreatedEvent,
        storage::{FileLocation, StorageTier},
        Filename, Medium, MediumCreateRequest, MediumId, MediumItemCreateRequest, MediumItemType,
        MediumType, Priority,
    },
    user::UserId,
};
use mime::Mime;
use tokio::io::AsyncRead;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::{
    error::{ApplicationError, ApplicationResult},
    event_bus::PublishEvent,
    medium::ports::FileStorage,
    user::QuotaManager,
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
    quota_manager: Arc<QuotaManager>,
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

        self.quota_manager
            .with_quota(command.user_id, command.file_size, || async {
                let medium_type = command
                    .medium_type
                    .unwrap_or_else(|| MediumType::from(command.mime_type.clone()));
                let filename = Filename::new(&command.filename)
                    .map_err(|e| ApplicationError::Domain { source: e })?;
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

                info!(
                    medium_id = %medium_id,
                    "Medium created successfully from stream"
                );

                Ok(medium_id)
            })
            .await
    }
}
