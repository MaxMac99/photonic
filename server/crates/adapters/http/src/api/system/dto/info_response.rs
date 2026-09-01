use serde::{Deserialize, Serialize};
use system::application::SystemInfo;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InfoResponse {
    pub version: String,
    pub client_id: Option<String>,
    pub token_url: Option<String>,
    pub authorize_url: Option<String>,
}

impl From<SystemInfo> for InfoResponse {
    fn from(info: SystemInfo) -> Self {
        Self {
            version: info.version,
            client_id: info.client_id,
            token_url: info.token_url,
            authorize_url: info.authorize_url,
        }
    }
}
