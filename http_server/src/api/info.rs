use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{error::Result, AppState};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub version: String,
    pub openid_configuration_url: String,
    pub client_id: String,
    pub authorize_url: String,
    pub token_url: String,
    pub response_types_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
}

pub async fn info(
    State(AppState { server_config, .. }): State<AppState>,
) -> Result<(StatusCode, Json<Info>)> {
    Ok((
        StatusCode::OK,
        Json::from(Info {
            version: String::from(VERSION),
            openid_configuration_url: server_config.openid_url.clone(),
            client_id: server_config.client_id.clone(),
            authorize_url: server_config
                .openid_configuration
                .authorization_endpoint
                .clone(),
            token_url: server_config.openid_configuration.token_endpoint.clone(),
            response_types_supported: server_config
                .openid_configuration
                .response_types_supported
                .clone(),
            scopes_supported: server_config.openid_configuration.scopes_supported.clone(),
            claims_supported: server_config.openid_configuration.claims_supported.clone(),
            code_challenge_methods_supported: server_config
                .openid_configuration
                .code_challenge_methods_supported
                .clone(),
        }),
    ))
}
