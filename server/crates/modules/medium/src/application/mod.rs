use std::sync::Arc;

use commands::PublishCleanupEvent;
use kernel::event_bus::PublishEvent;

use crate::{
    application::{
        ports::{FileStorage, MediumRepository, PublishMediumEvent, QuotaPort},
        queries::MediumQueryPort,
    },
    domain::{
        events::{MediumCreatedEvent, MediumItemCreatedEvent, MediumThumbhashUpdatedEvent},
        StoragePathService,
    },
};

pub mod commands;
pub mod listeners;
pub mod ports;
pub mod queries;

pub struct MediumApplicationHandlers {
    pub create_medium_stream: Arc<commands::CreateMediumStreamHandler>,
    pub add_medium_item: Arc<commands::AddMediumItemHandler>,
    pub find_all_media: Arc<queries::FindAllMediaHandler>,
    pub find_medium: Arc<queries::FindMediumHandler>,
    pub get_medium_file: Arc<queries::GetMediumFileHandler>,
    pub enrich_medium_with_metadata: Arc<commands::EnrichMediumWithMetadataHandler>,
    pub move_to_permanent_storage: Arc<commands::MoveToPermanentStorageHandler>,
    pub cleanup_expired_temp_storage: Arc<commands::CleanupExpiredTempStorageHandler>,
}

impl MediumApplicationHandlers {
    pub fn new(
        medium_repository: Arc<dyn MediumRepository>,
        medium_queries: Arc<dyn MediumQueryPort>,
        file_storage: Arc<dyn FileStorage>,
        quota: Arc<dyn QuotaPort>,
        event_bus: Arc<dyn PublishMediumEvent>,
        cleanup_event_bus: Arc<dyn PublishCleanupEvent>,
        storage_path_service: Arc<StoragePathService>,
        medium_event_bus: Arc<dyn PublishEvent<MediumCreatedEvent>>,
        item_event_bus: Arc<dyn PublishEvent<MediumItemCreatedEvent>>,
        thumbhash_event_bus: Arc<dyn PublishEvent<MediumThumbhashUpdatedEvent>>,
    ) -> Self {
        Self {
            create_medium_stream: Arc::new(commands::CreateMediumStreamHandler::new(
                file_storage.clone(),
                quota.clone(),
                medium_event_bus,
            )),
            add_medium_item: Arc::new(commands::AddMediumItemHandler::new(
                medium_repository.clone(),
                file_storage.clone(),
                quota,
                item_event_bus,
                thumbhash_event_bus,
            )),
            find_all_media: Arc::new(queries::FindAllMediaHandler::new(medium_queries.clone())),
            find_medium: Arc::new(queries::FindMediumHandler::new(medium_queries.clone())),
            get_medium_file: Arc::new(queries::GetMediumFileHandler::new(
                medium_queries,
                file_storage.clone(),
            )),
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
