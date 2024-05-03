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
}

pub async fn info(
    State(AppState { server_config, .. }): State<AppState>,
) -> Result<(StatusCode, Json<Info>)> {
    Ok((
        StatusCode::OK,
        Json::from(Info {
            version: String::from(VERSION),
            openid_configuration_url: server_config.openid_configuration_url.clone(),
            client_id: server_config.client_id.clone(),
        }),
    ))
}
