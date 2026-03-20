use std::sync::Arc;

use crate::config::AuthConfig;

pub mod queries;

pub use queries::{SystemInfo, SystemInfoHandler};

pub struct SystemApplicationHandlers {
    pub info: SystemInfoHandler,
}

impl SystemApplicationHandlers {
    pub fn new(auth_config: Arc<AuthConfig>) -> Self {
        Self {
            info: SystemInfoHandler::new(auth_config),
        }
    }
}
