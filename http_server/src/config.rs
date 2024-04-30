use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};

const ENV_OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
const ENV_OAUTH_JWKS_URL: &str = "OAUTH_JWKS_URL";
const ENV_OPENID_CONFIGURATION_URL: &str = "OPENID_CONFIGURATION_URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub client_id: String,
    pub jwks_url: String,
    pub openid_configuration_url: String,
}

impl ServerConfig {
    pub fn load() -> Result<Self, Whatever> {
        let client_id = std::env::var(ENV_OAUTH_CLIENT_ID)
            .whatever_context("Could not find oauth client id")?;
        let jwks_url =
            std::env::var(ENV_OAUTH_JWKS_URL).whatever_context("Could not find jwks url")?;
        let openid_configuration_url = std::env::var(ENV_OPENID_CONFIGURATION_URL)
            .whatever_context("Could not find openid configuration url")?;
        let config = Self {
            client_id,
            jwks_url,
            openid_configuration_url,
        };
        Ok(config)
    }
}
