use crate::{error::Result, state::AppState, system::model::InfoResponse};
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router(state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(system_info))
        .with_state(state)
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    tag = "system",
    responses(
        (status = 200, content_type = "application/json", description = "Info on the current system", body = InfoResponse),
    ),
)]
async fn system_info(State(state): State<AppState>) -> Result<(StatusCode, Json<InfoResponse>)> {
    Ok((
        StatusCode::OK,
        Json::from(InfoResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            client_id: state.config.server.client_id.clone(),
            token_url: state.config.server.token_url.clone(),
            authorize_url: state.config.server.authorize_url.clone(),
        }),
    ))
}
