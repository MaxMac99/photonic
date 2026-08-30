use std::sync::Arc;

use medium::application::MediumApplicationHandlers;
use metadata::application::MetadataApplicationHandlers;
use system::application::SystemApplicationHandlers;
use task::application::ProcessingApplicationHandlers;
use user::application::UserApplicationHandlers;

#[derive(Clone)]
pub struct AppState {
    pub user_handlers: Arc<UserApplicationHandlers>,
    pub medium_handlers: Arc<MediumApplicationHandlers>,
    pub metadata_handlers: Arc<MetadataApplicationHandlers>,
    pub system_handlers: Arc<SystemApplicationHandlers>,
    pub processing_handlers: Arc<ProcessingApplicationHandlers>,
}

impl AppState {
    pub fn new(
        user_handlers: Arc<UserApplicationHandlers>,
        medium_handlers: Arc<MediumApplicationHandlers>,
        metadata_handlers: Arc<MetadataApplicationHandlers>,
        system_handlers: Arc<SystemApplicationHandlers>,
        processing_handlers: Arc<ProcessingApplicationHandlers>,
    ) -> Self {
        Self {
            user_handlers,
            medium_handlers,
            metadata_handlers,
            system_handlers,
            processing_handlers,
        }
    }
}
