use std::error::Error;
use std::fmt::Debug;

use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use snafu::{AsErrorSource, ErrorCompat, prelude::*, Snafu};
use tracing::log::error;

use fotonic::service::CreateMediumError;

trait BacktraceError: ErrorCompat + Error + AsErrorSource + Debug {}

impl<T: ErrorCompat + Error + AsErrorSource + Debug> BacktraceError for T {}

#[derive(Snafu, Debug)]
pub enum ResponseError {
    #[snafu(display("{message}"))]
    InvalidArgument {
        message: String,
    },
    #[snafu(display("Authentication required"))]
    AuthenticationRequired,
    #[snafu(display("{message}"))]
    PermissionDenied {
        message: String,
    },
    #[snafu(display("{message}"))]
    NotFound {
        message: String,
    },
    #[snafu(display("{message}"))]
    AlreadyExists {
        message: String,
    },
    #[snafu(display("Internal error"))]
    Internal {
        message: String,
        source: Box<dyn BacktraceError>,
    },
}

impl From<CreateMediumError> for ResponseError {
    fn from(err: CreateMediumError) -> Self {
        match err {
            CreateMediumError::NoDateTaken { .. } => ResponseError::NotFound { message: err.to_string() },
            CreateMediumError::WrongAlbum { .. } => ResponseError::NotFound { message: err.to_string() },
            _ => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            }
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        match self {
            ResponseError::NotFound { message } => (StatusCode::NOT_FOUND, Json(message)).into_response(),
            ResponseError::AuthenticationRequired => StatusCode::UNAUTHORIZED.into_response(),
            ResponseError::PermissionDenied { message } => (StatusCode::FORBIDDEN, Json(message)).into_response(),
            ResponseError::InvalidArgument { message } => (StatusCode::BAD_REQUEST, Json(message)).into_response(),
            ResponseError::AlreadyExists { message } => (StatusCode::CONFLICT, Json(message)).into_response(),
            ResponseError::Internal { ref message, ref source } => {
                if let Some(backtrace) = source.backtrace() {
                    error!("Internal server error: {}\n{}", source.to_string(), backtrace);
                } else {
                    error!("Internal server error: {}", source.to_string());
                }
                (StatusCode::INTERNAL_SERVER_ERROR, Json("Something went wrong")).into_response()
            }
        }
    }
}
