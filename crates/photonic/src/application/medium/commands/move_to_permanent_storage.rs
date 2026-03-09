use std::sync::Arc;

use derive_new::new;
use tracing::{debug, error, info, instrument};

use crate::{
    application::{
        error::ApplicationResult,
        medium::ports::{FileStorage, MediumRepository},
    },
    domain::{
        error::format_error_with_backtrace as format_domain_error,
        medium::{
            storage::{FileLocation, StorageTier},
            MediumId, MediumItemId,
        },
        user::UserId,
    },
    infrastructure::storage::filesystem::path_service::StoragePathService,
};

pub struct MoveToPermanentStorageCommand {
    pub medium_id: MediumId,
    pub user_id: UserId,
}

#[derive(new)]
pub struct MoveToPermanentStorageHandler {
    medium_repository: Arc<dyn MediumRepository>,
    file_storage: Arc<dyn FileStorage>,
    storage_path_service: Arc<StoragePathService>,
}

struct CopyOperation {
    item_id: MediumItemId,
    src: FileLocation,
    dest: FileLocation,
}

impl MoveToPermanentStorageHandler {
    #[instrument(skip(self, command), fields(
        medium_id = %command.medium_id,
        user_id = %command.user_id,
    ))]
    pub async fn handle(&self, command: MoveToPermanentStorageCommand) -> ApplicationResult<()> {
        let mut medium = match self
            .medium_repository
            .find_by_id(command.medium_id, command.user_id)
            .await?
        {
            Some(medium) => medium,
            None => {
                debug!("Medium not found, skipping move to permanent storage");
                return Ok(());
            }
        };

        // Pass 1: Determine which items need copying and compute destinations
        let operations: Vec<CopyOperation> = medium
            .items
            .iter()
            .filter_map(|item| {
                // Skip if item already has a permanent location
                let has_permanent = item
                    .locations
                    .iter()
                    .any(|l| l.storage_tier == StorageTier::Permanent);
                if has_permanent {
                    return None;
                }

                let temp_location = item
                    .locations
                    .iter()
                    .find(|l| l.storage_tier == StorageTier::Temporary)?;

                let permanent_path = self
                    .storage_path_service
                    .generate_permanent_path(&medium, item);

                Some(CopyOperation {
                    item_id: item.id,
                    src: temp_location.clone(),
                    dest: FileLocation::permanent(permanent_path),
                })
            })
            .collect();

        if operations.is_empty() {
            debug!("No items to copy, all already in permanent storage");
            return Ok(());
        }

        // Pass 2: Copy files on disk (temp files remain for other jobs)
        let mut completed_copies: Vec<&CopyOperation> = Vec::new();
        for op in &operations {
            debug!(
                item_id = %op.item_id,
                src = ?op.src.relative_path,
                dest = ?op.dest.relative_path,
                "Copying medium item to permanent storage"
            );

            if let Err(copy_err) = self.file_storage.copy_file(&op.src, &op.dest).await {
                error!(
                    item_id = %op.item_id,
                    error = %format_domain_error(&copy_err),
                    "Failed to copy file to permanent storage, rolling back"
                );
                // Rollback: delete already-copied files
                for completed in &completed_copies {
                    if let Err(rollback_err) =
                        self.file_storage.delete_file(&completed.dest).await
                    {
                        error!(
                            item_id = %completed.item_id,
                            error = %format_domain_error(&rollback_err),
                            "CRITICAL: Failed to delete copied file during rollback. Manual cleanup required"
                        );
                    }
                }
                return Err(copy_err.into());
            }
            completed_copies.push(op);
        }

        // Pass 3: Update domain entities — add permanent location alongside temp
        for op in &operations {
            if let Some(item) = medium.find_item_mut(op.item_id) {
                item.add_location(op.dest.clone());
            }
        }

        // Pass 4: Persist
        if let Err(save_err) = self.medium_repository.save(&medium).await {
            error!(
                medium_id = %command.medium_id,
                error = %format_domain_error(&save_err),
                "Failed to save medium after copying files, rolling back"
            );
            // Rollback: delete all copied files
            for op in &operations {
                if let Err(rollback_err) =
                    self.file_storage.delete_file(&op.dest).await
                {
                    error!(
                        item_id = %op.item_id,
                        error = %format_domain_error(&rollback_err),
                        "CRITICAL: Failed to delete copied file during rollback. Manual cleanup required"
                    );
                }
            }
            return Err(save_err.into());
        }

        info!(
            medium_id = %command.medium_id,
            items_copied = operations.len(),
            "Medium items copied to permanent storage"
        );

        Ok(())
    }
}