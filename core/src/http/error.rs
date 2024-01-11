use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use tracing::log::debug;

use crate::errors;

impl IntoResponse for errors::Error {
    fn into_response(self) -> Response {
        match self {
            errors::Error::Internal(desc) => {
                debug!("Internal Server Error: ");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(desc)).into_response()
            }
            errors::Error::NotFound(desc) => (StatusCode::NOT_FOUND, Json(desc)).into_response(),
            errors::Error::AuthenticationRequired => StatusCode::UNAUTHORIZED.into_response(),
            errors::Error::PermissionDenied(desc) => (StatusCode::FORBIDDEN, Json(desc)).into_response(),
            errors::Error::InvalidArgument(desc) => (StatusCode::BAD_REQUEST, Json(desc)).into_response(),
            errors::Error::AlreadyExists(desc) => (StatusCode::CONFLICT, Json(desc)).into_response(),
        }
    }
}
