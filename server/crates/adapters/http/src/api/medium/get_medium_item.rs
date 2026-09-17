use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
};
use jwt_authorizer::JwtClaims;
use medium::application::queries::{GetMediumItemFileQuery, MediumFile};
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    api::{error::ApiResult, router::Binary, state::AppState},
    auth::JwtUserClaims,
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}/item/{item_id}/raw",
    tag = "medium",
    responses(
        (status = 200, description = "The raw file", body = Binary, content_type = "*/*", headers(
            ("content-type" = String)
        )),
    ),
)]
pub async fn get_medium_item(
    State(state): State<AppState>,
    Path((medium_id, item_id)): Path<(Uuid, Uuid)>,
    JwtClaims(user): JwtClaims<JwtUserClaims>,
) -> ApiResult<(StatusCode, HeaderMap, Body)> {
    let user_id = user.user_id();
    info!(user_id = %user_id, medium_id = %medium_id, item_id = %item_id, "Fetching medium item");

    let query = GetMediumItemFileQuery {
        user_id,
        medium_id,
        item_id,
    };

    match state
        .medium_handlers
        .get_medium_file
        .get_item_file(query)
        .await
    {
        Ok(file) => Ok(file_to_response(file)),
        Err(e) => {
            error!(
                error = %kernel::app_error::format_error_with_backtrace(&e),
                "Failed to retrieve medium item file"
            );
            Err(e.into())
        }
    }
}

pub(crate) fn file_to_response(file: MediumFile) -> (StatusCode, HeaderMap, Body) {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_str(file.mime.as_ref())
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    (
        StatusCode::OK,
        headers,
        Body::from_stream(tokio_util::io::ReaderStream::new(file.stream)),
    )
}
