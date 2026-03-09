use std::sync::Arc;

use crate::infrastructure::config::GlobalConfig;

pub mod queries;

pub use queries::{SystemInfo, SystemInfoHandler};

pub struct SystemApplicationHandlers {
    pub info: SystemInfoHandler,
}

impl SystemApplicationHandlers {
    pub fn new(config: Arc<GlobalConfig>) -> Self {
        Self {
            info: SystemInfoHandler::new(config),
        }
    }
}
