use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};

const ENV_OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
const ENV_OAUTH_JWKS_URL: &str = "OAUTH_JWKS_URL";
const ENV_OAUTH_AUTHORIZE_URL: &str = "OAUTH_AUTHORIZE_URL";
const ENV_OAUTH_TOKEN_URL: &str = "OAUTH_TOKEN_URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub client_id: String,
    pub jwks_url: String,
    pub authorize_url: String,
    pub token_url: String,
}

impl ServerConfig {
    pub fn load() -> Result<Self, Whatever> {
        let client_id = std::env::var(ENV_OAUTH_CLIENT_ID)
            .whatever_context("Could not find oauth client id")?;
        let jwks_url =
            std::env::var(ENV_OAUTH_JWKS_URL).whatever_context("Could not find jwks url")?;
        let authorize_url = std::env::var(ENV_OAUTH_AUTHORIZE_URL)
            .whatever_context("Could not find authorize url")?;
        let token_url =
            std::env::var(ENV_OAUTH_TOKEN_URL).whatever_context("Could not find token url")?;
        let config = Self {
            client_id,
            jwks_url,
            authorize_url,
            token_url,
        };
        Ok(config)
    }
}
