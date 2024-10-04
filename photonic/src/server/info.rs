use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{error::Result, server::AppState};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub version: String,
    pub openid_configuration_url: String,
    pub client_id: String,
    pub authorize_url: String,
    pub token_url: String,
}

pub async fn info(
    State(AppState { config, .. }): State<AppState>,
) -> Result<(StatusCode, Json<Info>)> {
    Ok((
        StatusCode::OK,
        Json::from(Info {
            version: String::from(VERSION),
            openid_configuration_url: config.oauth.openid_configuration_url.clone(),
            client_id: config.oauth.client_id.clone(),
            authorize_url: config.oauth.authorize_url.clone(),
            token_url: config.oauth.token_url.clone(),
        }),
    ))
}
