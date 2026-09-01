use std::sync::Arc;

mod config;
pub mod queries;

pub use config::AuthConfig;
pub use queries::{SystemInfo, SystemInfoHandler};

pub struct SystemApplicationHandlers {
    pub info: SystemInfoHandler,
}

impl SystemApplicationHandlers {
    pub fn new(auth_config: Option<Arc<AuthConfig>>) -> Self {
        Self {
            info: SystemInfoHandler::new(auth_config),
        }
    }
}
