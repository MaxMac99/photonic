use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{AppState, error::Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "authorizeUrl")]
    pub authorize_url: String,
    #[serde(rename = "tokenUrl")]
    pub token_url: String,
}

pub async fn info(
    State(AppState { server_config, .. }): State<AppState>,
) -> Result<(StatusCode, Json<Info>)> {
    Ok((
        StatusCode::OK,
        Json::from(Info {
            client_id: server_config.client_id.clone(),
            authorize_url: server_config.authorize_url.clone(),
            token_url: server_config.token_url.clone(),
        }),
    ))
}
