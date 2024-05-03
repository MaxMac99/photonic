use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};

const ENV_OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
const ENV_OPENID_CONFIGURATION_URL: &str = "OPENID_CONFIGURATION_URL";

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub client_id: String,
    pub openid_url: String,
    pub openid_configuration: OpenIdDiscovery,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenIdDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub jwks_uri: String,
    pub scopes_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
}

impl ServerConfig {
    pub async fn load() -> Result<Self, Whatever> {
        let client_id = std::env::var(ENV_OAUTH_CLIENT_ID)
            .whatever_context("Could not find oauth client id")?;
        let openid_configuration_url = std::env::var(ENV_OPENID_CONFIGURATION_URL)
            .whatever_context("Could not find openid configuration url")?;

        let discovery = reqwest::get(openid_configuration_url.clone())
            .await
            .whatever_context("Could not send request to openid")?
            .json::<OpenIdDiscovery>()
            .await
            .whatever_context("Could not send request to openid")?;

        let config = Self {
            client_id,
            openid_url: openid_configuration_url,
            openid_configuration: discovery,
        };
        Ok(config)
    }
}
