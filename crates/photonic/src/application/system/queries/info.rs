use std::sync::Arc;

use derive_new::new;

use crate::infrastructure::config::GlobalConfig;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub version: String,
    pub client_id: String,
    pub token_url: String,
    pub authorize_url: String,
}

#[derive(new)]
pub struct SystemInfoHandler {
    config: Arc<GlobalConfig>,
}

impl SystemInfoHandler {
    pub async fn handle(&self) -> SystemInfo {
        SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            client_id: self.config.server.client_id.clone(),
            token_url: self.config.server.token_url.clone(),
            authorize_url: self.config.server.authorize_url.clone(),
        }
    }
}
