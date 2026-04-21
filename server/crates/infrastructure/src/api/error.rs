use application::error::{format_error_with_backtrace, ApplicationError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::error::DomainError;
use serde_json::json;
use tracing::warn;

pub type ApiResult<T> = Result<T, ApiError>;

/// Newtype wrapper to implement IntoResponse for ApplicationError
/// (orphan rule: both types are external to this crate)
pub struct ApiError(pub ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        Self(err)
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        Self(ApplicationError::Domain { source: err })
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            ApplicationError::Domain { source } => match source {
                DomainError::EntityNotFound { .. } => (StatusCode::NOT_FOUND, source.to_string()),
                DomainError::QuotaExceeded { .. } => (StatusCode::FORBIDDEN, source.to_string()),
                DomainError::Validation { .. } => (StatusCode::BAD_REQUEST, source.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, source.to_string()),
            },
            ApplicationError::Repository { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string())
            }
            ApplicationError::ExternalService { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, self.0.to_string())
            }
            ApplicationError::Internal { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string())
            }
            ApplicationError::Conflict { .. } => (StatusCode::CONFLICT, self.0.to_string()),
        };

        warn!(
            error = %format_error_with_backtrace(&self.0),
            status_code = ?status,
            "Application error converted to HTTP response"
        );

        (status, Json(json!({ "error": message }))).into_response()
    }
}
