use std::sync::Arc;

use crate::{
    application::{
        medium::ports::{FileStorage, MediumRepository, PublishMediumEvent},
        user::QuotaManager,
    },
    infrastructure::storage::filesystem::path_service::StoragePathService,
};

use commands::PublishCleanupEvent;

pub mod commands;
pub mod listeners;
pub mod ports;
pub mod queries;

pub struct MediumApplicationHandlers {
    pub create_medium_stream: Arc<commands::CreateMediumStreamHandler>,
    pub find_all_media: Arc<queries::FindAllMediaHandler>,
    pub find_medium: Arc<queries::FindMediumHandler>,
    pub enrich_medium_with_metadata: Arc<commands::EnrichMediumWithMetadataHandler>,
    pub move_to_permanent_storage: Arc<commands::MoveToPermanentStorageHandler>,
    pub cleanup_expired_temp_storage: Arc<commands::CleanupExpiredTempStorageHandler>,
}

impl MediumApplicationHandlers {
    pub fn new(
        medium_repository: Arc<dyn MediumRepository>,
        file_storage: Arc<dyn FileStorage>,
        quota_manager: Arc<QuotaManager>,
        event_bus: Arc<dyn PublishMediumEvent>,
        cleanup_event_bus: Arc<dyn PublishCleanupEvent>,
        storage_path_service: Arc<StoragePathService>,
    ) -> Self {
        Self {
            create_medium_stream: Arc::new(commands::CreateMediumStreamHandler::new(
                medium_repository.clone(),
                file_storage.clone(),
                quota_manager,
                event_bus.clone(),
            )),
            find_all_media: Arc::new(queries::FindAllMediaHandler::new(medium_repository.clone())),
            find_medium: Arc::new(queries::FindMediumHandler::new(medium_repository.clone())),
            enrich_medium_with_metadata: Arc::new(commands::EnrichMediumWithMetadataHandler::new(
                medium_repository.clone(),
                event_bus,
            )),
            move_to_permanent_storage: Arc::new(commands::MoveToPermanentStorageHandler::new(
                medium_repository.clone(),
                file_storage.clone(),
                storage_path_service,
            )),
            cleanup_expired_temp_storage: Arc::new(
                commands::CleanupExpiredTempStorageHandler::new(
                    medium_repository,
                    file_storage,
                    cleanup_event_bus,
                ),
            ),
        }
    }
}