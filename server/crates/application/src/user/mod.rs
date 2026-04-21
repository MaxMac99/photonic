use std::sync::Arc;

use crate::{config::QuotaConfig, user::ports::PublishUserEvent};

pub mod commands;
pub mod ports;
pub mod quota_manager;

pub use ports::UserRepository;
pub use quota_manager::QuotaManager;

pub struct UserApplicationHandlers {
    pub user_exists: Arc<commands::EnsureUserExistsHandler>,
}

impl UserApplicationHandlers {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        event_bus: Arc<dyn PublishUserEvent>,
        quota_config: Arc<QuotaConfig>,
    ) -> Self {
        Self {
            user_exists: Arc::new(commands::EnsureUserExistsHandler::new(
                user_repository,
                event_bus,
                quota_config,
            )),
        }
    }
}
