use std::sync::Arc;

use derive_new::new;

use crate::application::AuthConfig;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub version: String,
    pub client_id: Option<String>,
    pub token_url: Option<String>,
    pub authorize_url: Option<String>,
}

#[derive(new)]
pub struct SystemInfoHandler {
    auth_config: Option<Arc<AuthConfig>>,
}

impl SystemInfoHandler {
    pub async fn handle(&self) -> SystemInfo {
        SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            client_id: self.auth_config.as_ref().map(|c| c.client_id.clone()),
            token_url: self.auth_config.as_ref().map(|c| c.token_url.clone()),
            authorize_url: self.auth_config.as_ref().map(|c| c.authorize_url.clone()),
        }
    }
}
