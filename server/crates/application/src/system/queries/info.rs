use std::sync::Arc;

use derive_new::new;

use crate::config::AuthConfig;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub version: String,
    pub client_id: String,
    pub token_url: String,
    pub authorize_url: String,
}

#[derive(new)]
pub struct SystemInfoHandler {
    auth_config: Arc<AuthConfig>,
}

impl SystemInfoHandler {
    pub async fn handle(&self) -> SystemInfo {
        SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            client_id: self.auth_config.client_id.clone(),
            token_url: self.auth_config.token_url.clone(),
            authorize_url: self.auth_config.authorize_url.clone(),
        }
    }
}
