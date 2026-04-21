use std::sync::Arc;

use crate::metadata::{
    commands::ExtractMetadataHandler,
    ports::{MetadataExtractor, MetadataRepository, PublishMetadataEvent},
    queries::FindMetadataByMediumIdHandler,
};

pub mod commands;
pub mod listeners;
pub mod ports;
pub mod queries;

pub struct MetadataApplicationHandlers {
    pub extract_metadata_handler: Arc<ExtractMetadataHandler>,
    pub find_metadata_by_medium_id: Arc<FindMetadataByMediumIdHandler>,
}

impl MetadataApplicationHandlers {
    pub fn new(
        metadata_extractor: Arc<dyn MetadataExtractor>,
        metadata_repository: Arc<dyn MetadataRepository>,
        event_bus: Arc<dyn PublishMetadataEvent>,
    ) -> Self {
        Self {
            extract_metadata_handler: Arc::new(ExtractMetadataHandler::new(
                metadata_extractor,
                metadata_repository.clone(),
                event_bus,
            )),
            find_metadata_by_medium_id: Arc::new(FindMetadataByMediumIdHandler::new(
                metadata_repository,
            )),
        }
    }
}
