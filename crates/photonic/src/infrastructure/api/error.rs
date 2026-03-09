use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::warn;

use crate::{
    application::error::{format_error_with_backtrace, ApplicationError},
    domain::error::DomainError,
};

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApplicationError::Domain { source } => match source {
                DomainError::EntityNotFound { .. } => (StatusCode::NOT_FOUND, source.to_string()),
                DomainError::QuotaExceeded { .. } => (StatusCode::FORBIDDEN, source.to_string()),
                DomainError::Validation { .. } => (StatusCode::BAD_REQUEST, source.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, source.to_string()),
            },
            ApplicationError::Repository { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ApplicationError::ExternalService { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
        };

        warn!(
            error = %format_error_with_backtrace(&self),
            status_code = ?status,
            "Application error converted to HTTP response"
        );

        (status, Json(json!({ "error": message }))).into_response()
    }
}
