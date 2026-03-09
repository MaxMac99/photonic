use std::sync::Arc;

use snafu::Whatever;

use crate::{
    application::{
        medium::MediumApplicationHandlers, metadata::MetadataApplicationHandlers,
        system::SystemApplicationHandlers, user::UserApplicationHandlers,
    },
    infrastructure::{config::GlobalConfig, di::Container},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GlobalConfig>,
    pub user_handlers: Arc<UserApplicationHandlers>,
    pub medium_handlers: Arc<MediumApplicationHandlers>,
    pub metadata_handlers: Arc<MetadataApplicationHandlers>,
    pub system_handlers: Arc<SystemApplicationHandlers>,
}

impl AppState {
    pub async fn new(container: Arc<Container>) -> Result<Self, Whatever> {
        Ok(Self {
            config: container.config(),
            user_handlers: container.user_handlers(),
            medium_handlers: container.medium_handlers(),
            metadata_handlers: container.metadata_handlers(),
            system_handlers: container.system_handlers(),
        })
    }
}
