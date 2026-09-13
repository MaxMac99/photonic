use std::sync::Arc;

use byte_unit::Byte;
use derive_new::new;
use kernel::{
    app_error::{ApplicationError, ApplicationResult},
    error::{EntityNotFoundSnafu, ValidationSnafu},
    event_bus::PublishEvent,
    Dimensions, FileLocation, Filename, MediumId, MediumItemId, Priority, StorageTier, UserId,
};
use mime::Mime;
use snafu::OptionExt;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    application::ports::{FileStorage, MediumRepository, QuotaPort},
    domain::{
        events::{MediumItemCreatedEvent, MediumThumbhashUpdatedEvent},
        Medium, MediumItemCreateRequest, MediumItemType, Thumbhash, ThumbnailVariant,
    },
};

/// Upper bound for a single uploaded thumbnail. Thumbnails are
/// client-generated images of at most 1440px; anything larger is invalid.
const MAX_THUMBNAIL_SIZE: u64 = 10 * 1024 * 1024;

pub struct AddMediumItemCommand {
    pub user_id: UserId,
    pub medium_id: MediumId,
    pub item_type: MediumItemType,
    pub variant: Option<ThumbnailVariant>,
    pub mime_type: Mime,
    pub filename: String,
    pub filesize: Byte,
    pub priority: Option<i32>,
    pub dimensions: Option<Dimensions>,
    pub data: Vec<u8>,
}

#[derive(new)]
pub struct AddMediumItemHandler {
    medium_repository: Arc<dyn MediumRepository>,
    file_storage: Arc<dyn FileStorage>,
    quota: Arc<dyn QuotaPort>,
    event_bus: Arc<dyn PublishEvent<MediumItemCreatedEvent>>,
    thumbhash_event_bus: Arc<dyn PublishEvent<MediumThumbhashUpdatedEvent>>,
}

impl AddMediumItemHandler {
    #[instrument(skip(self, command), fields(
        user_id = %command.user_id,
        medium_id = %command.medium_id,
        item_type = ?command.item_type,
        filesize = command.filesize.as_u64(),
    ))]
    pub async fn handle(&self, command: AddMediumItemCommand) -> ApplicationResult<MediumItemId> {
        info!("Adding medium item");

        if command.item_type == MediumItemType::Preview {
            Self::validate_preview(&command)?;
        }

        let data_len = command.data.len() as u64;
        if data_len != command.filesize.as_u64() {
            return Err(ApplicationError::Domain {
                source: ValidationSnafu {
                    message: format!(
                        "Declared filesize {} does not match payload size {data_len}",
                        command.filesize.as_u64()
                    ),
                }
                .build(),
            });
        }

        let reservation = self
            .quota
            .reserve(command.user_id, command.filesize)
            .await?;

        match self.add(command).await {
            Ok(item_id) => {
                if let Err(e) = self.quota.commit(reservation).await {
                    warn!(error = %e, "Failed to commit quota reservation");
                }
                Ok(item_id)
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

    fn validate_preview(command: &AddMediumItemCommand) -> ApplicationResult<()> {
        let variant = command.variant.ok_or_else(|| ApplicationError::Domain {
            source: ValidationSnafu {
                message: "thumbnail variant is required for preview items".to_string(),
            }
            .build(),
        })?;

        let dimensions = command.dimensions.ok_or_else(|| ApplicationError::Domain {
            source: ValidationSnafu {
                message: "dimensions are required for preview items".to_string(),
            }
            .build(),
        })?;

        if command.mime_type.type_() != mime::IMAGE {
            return Err(ApplicationError::Domain {
                source: ValidationSnafu {
                    message: format!(
                        "Preview items must be images, got '{}'",
                        command.mime_type.essence_str()
                    ),
                }
                .build(),
            });
        }

        if command.filesize.as_u64() > MAX_THUMBNAIL_SIZE {
            return Err(ApplicationError::Domain {
                source: ValidationSnafu {
                    message: format!(
                        "Thumbnail exceeds maximum size of {MAX_THUMBNAIL_SIZE} bytes"
                    ),
                }
                .build(),
            });
        }

        variant
            .validate_dimensions(dimensions.width(), dimensions.height())
            .map_err(|e| ApplicationError::Domain { source: e })
    }

    async fn add(&self, command: AddMediumItemCommand) -> ApplicationResult<MediumItemId> {
        let mut medium = self
            .medium_repository
            .find_by_id(command.medium_id, command.user_id)
            .await?
            .context(EntityNotFoundSnafu {
                entity: "Medium",
                id: command.medium_id,
            })?;

        let filename =
            Filename::new(&command.filename).map_err(|e| ApplicationError::Domain { source: e })?;
        let priority = command.priority.map(Priority::new).unwrap_or_default();

        let cache_file_id = Uuid::new_v4();
        let extension = filename.extension();
        let location = FileLocation::new(
            StorageTier::Cache,
            std::path::PathBuf::from(format!("{cache_file_id}.{extension}")),
        );

        let item_request = MediumItemCreateRequest {
            owner_id: command.user_id,
            medium_item_type: command.item_type,
            mime: command.mime_type,
            filename,
            filesize: command.filesize,
            priority,
            dimensions: command.dimensions,
            locations: vec![location.clone()],
        };

        let event = medium.add_item(item_request)?;

        // The tiny thumbnail doubles as the thumbhash source; a thumbnail
        // that cannot be decoded is invalid, so validation is enforced here.
        let thumbhash = if command.variant == Some(ThumbnailVariant::Tiny) {
            Some(
                Thumbhash::compute(&command.data)
                    .map_err(|e| ApplicationError::Domain { source: e })?,
            )
        } else {
            None
        };

        debug!(
            medium_id = %medium.id,
            location = ?location.relative_path,
            "Storing thumbnail and persisting event"
        );

        if let Err(e) = self.file_storage.store_file(&location, command.data).await {
            error!(
                medium_id = %medium.id,
                error = %e,
                "File storage failed for medium item"
            );
            return Err(ApplicationError::Domain { source: e });
        }

        if let Err(e) = self.event_bus.publish(event).await {
            error!(
                medium_id = %medium.id,
                error = %e,
                "Failed to publish MediumItemCreatedEvent"
            );
            // Best-effort cleanup of the orphaned cache file.
            let _ = self.file_storage.delete_file(&location).await;
            return Err(e);
        }

        if let Some(hash) = thumbhash {
            self.publish_thumbhash(&medium, hash).await;
        }

        info!(medium_id = %medium.id, "Medium item added successfully");
        // The item added last in the aggregate is the one we just created.
        Ok(medium.items.last().map(|i| i.id).unwrap_or_default())
    }

    async fn publish_thumbhash(&self, medium: &Medium, hash: Thumbhash) {
        let mut event = MediumThumbhashUpdatedEvent::new(medium.owner_id, medium.id, hash);
        event.metadata.expected_version = medium.version;
        if let Err(e) = self.thumbhash_event_bus.publish(event).await {
            warn!(
                medium_id = %medium.id,
                error = %e,
                "Failed to publish MediumThumbhashUpdatedEvent"
            );
        }
    }
}
