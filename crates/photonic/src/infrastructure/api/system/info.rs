use axum::{debug_handler, extract::State, http::StatusCode, Json};
use tracing::instrument;

use crate::{
    application::error::ApplicationResult,
    infrastructure::api::{state::AppState, system::dto::InfoResponse},
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    tag = "system",
    responses(
        (status = 200, content_type = "application/json", description = "Info on the current system", body = InfoResponse),
    ),
)]
pub async fn system_info(
    State(state): State<AppState>,
) -> ApplicationResult<(StatusCode, Json<InfoResponse>)> {
    let app_response = state.system_handlers.info.handle().await;

    let response: InfoResponse = app_response.into();

    Ok((StatusCode::OK, Json::from(response)))
}
