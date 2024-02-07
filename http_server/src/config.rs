use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};

const ENV_OAUTH_JWKS_URL: &str = "OAUTH_JWKS_URL";
const ENV_OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub jwks_url: String,
    pub client_id: String,
}

impl ServerConfig {
    pub fn load() -> Result<Self, Whatever> {
        let jwks_url = std::env::var(ENV_OAUTH_JWKS_URL)
            .whatever_context("Could not find jwks url")?;
        let client_id = std::env::var(ENV_OAUTH_CLIENT_ID)
            .whatever_context("Could not find oauth client id")?;
        let config = Self {
            jwks_url,
            client_id,
        };
        Ok(config)
    }
}
