use std::{path::PathBuf, sync::Arc};

use byte_unit::Byte;
use chrono::{DateTime, FixedOffset};
use derive_new::new;
use domain::{
    error::format_error_with_backtrace as format_domain_error,
    medium::{
        storage::{FileLocation, StorageTier},
        Filename, Medium, MediumCreateRequest, MediumId, MediumItemCreateRequest, MediumItemType,
        MediumType, Priority,
    },
    user::UserId,
};
use mime::Mime;
use tokio::{io::AsyncRead, join};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    error::{ApplicationError, ApplicationResult},
    medium::ports::{FileStorage, MediumRepository, PublishMediumEvent},
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
    medium_repository: Arc<dyn MediumRepository>,
    file_storage: Arc<dyn FileStorage>,
    quota_manager: Arc<QuotaManager>,
    event_bus: Arc<dyn PublishMediumEvent>,
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
                let medium_type = command.medium_type.unwrap_or_else(|| {
                    MediumType::from(command.mime_type.clone())
                });
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
                let (medium, medium_created_event, item_created_event) = Medium::new(medium_request)?;
                let medium_id = medium.id;

                debug!(
                    medium_id = %medium_id,
                    temp_location = ?temp_location.relative_path,
                    "Executing parallel file storage and database save"
                );

                let store_future = self.file_storage.store_file_stream(&temp_location, command.stream);
                let save_future = self.medium_repository.save(&medium);

                let (store_result, save_result) = join!(store_future, save_future);

                match (store_result, save_result) {
                    (Ok(_), Ok(_)) => {
                        debug!(
                            medium_id = %medium_id,
                            "File storage and database save both completed successfully"
                        );
                    }
                    (Err(store_err), Ok(_)) => {
                        error!(
                            medium_id = %medium_id,
                            error = %format_domain_error(&store_err),
                            "File storage failed but database save succeeded, cleaning up repository"
                        );
                        if let Err(delete_err) = self.medium_repository.delete(medium_id, command.user_id).await {
                            error!(
                                medium_id = %medium_id,
                                error = %format_domain_error(&delete_err),
                                "CRITICAL: Failed to delete medium from repository after storage failure. Manual cleanup required"
                            );
                        }
                        return Err(ApplicationError::Domain { source: store_err });
                    }
                    (Ok(_), Err(save_err)) => {
                        error!(
                            medium_id = %medium_id,
                            path = ?temp_location.relative_path,
                            error = %format_domain_error(&save_err),
                            "Database save failed but file storage succeeded, cleaning up file"
                        );
                        if let Err(cleanup_err) = self.file_storage.delete_file(&temp_location).await {
                            error!(
                                path = ?temp_location.relative_path,
                                error = %format_domain_error(&cleanup_err),
                                "CRITICAL: Failed to delete orphaned file. Manual cleanup required"
                            );
                        }
                        return Err(ApplicationError::Domain { source: save_err });
                    }
                    (Err(store_err), Err(save_err)) => {
                        error!(
                            medium_id = %medium_id,
                            store_error = %format_domain_error(&store_err),
                            save_error = %format_domain_error(&save_err),
                            "Both file storage and database save failed"
                        );
                        return Err(ApplicationError::Domain { source: store_err });
                    }
                }

                if let Err(e) = self.event_bus.publish(medium_created_event).await {
                    warn!(
                        medium_id = %medium_id,
                        error = %e,
                        "Failed to publish MediumCreatedEvent"
                    );
                }

                if let Err(e) = self.event_bus.publish(item_created_event).await {
                    warn!(
                        medium_id = %medium_id,
                        error = %e,
                        "Failed to publish MediumItemCreatedEvent"
                    );
                }

                info!(
                    medium_id = %medium_id,
                    "Medium created successfully from stream"
                );

                Ok(medium_id)
            })
            .await
    }
}
