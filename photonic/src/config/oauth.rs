use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};

const ENV_OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
const ENV_OAUTH_JWKS_URL: &str = "OAUTH_JWKS_URL";
const ENV_OPENID_CONFIGURATION_URL: &str = "OPENID_CONFIGURATION_URL";
const ENV_OAUTH_AUTHORIZE_URL: &str = "OAUTH_AUTHORIZE_URL";
const ENV_OAUTH_TOKEN_URL: &str = "OAUTH_TOKEN_URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth {
    pub client_id: String,
    pub jwks_url: String,
    pub openid_configuration_url: String,
    pub authorize_url: String,
    pub token_url: String,
}

impl OAuth {
    pub fn load() -> Result<Self, Whatever> {
        let client_id = std::env::var(ENV_OAUTH_CLIENT_ID)
            .whatever_context("Could not find oauth client id")?;
        let jwks_url =
            std::env::var(ENV_OAUTH_JWKS_URL).whatever_context("Could not find jwks url")?;

        let openid_configuration_url = std::env::var(ENV_OPENID_CONFIGURATION_URL)
            .whatever_context("Could not find openid configuration url")?;
        let authorize_url = std::env::var(ENV_OAUTH_AUTHORIZE_URL)
            .whatever_context("Could not find openid authorize url")?;
        let token_url = std::env::var(ENV_OAUTH_TOKEN_URL)
            .whatever_context("Could not find openid token url")?;

        Ok(Self {
            client_id,
            jwks_url,
            openid_configuration_url,
            authorize_url,
            token_url,
        })
    }
}
