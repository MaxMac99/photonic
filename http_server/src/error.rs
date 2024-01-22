use std::{error::Error, fmt::Debug};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use snafu::{AsErrorSource, ErrorCompat, Snafu};
use tracing::log::error;

use fotonic::service::{CreateMediumError, MediumRepoError};

pub(crate) trait BacktraceError:
    ErrorCompat + Error + AsErrorSource + Debug
{
}

impl<T: ErrorCompat + Error + AsErrorSource + Debug> BacktraceError for T {}

#[derive(Snafu, Debug)]
pub(crate) enum ResponseError {
    #[snafu(display("{message}"))]
    BadRequest { message: String },
    #[snafu(display("Authentication required"))]
    AuthenticationRequired,
    #[snafu(display("{message}"))]
    PermissionDenied { message: String },
    #[snafu(display("{message}"))]
    NotFound { message: String },
    #[snafu(display("{message}"))]
    AlreadyExists { message: String },
    #[snafu(display("Internal error"))]
    Internal {
        message: String,
        source: Box<dyn BacktraceError>,
    },
}

impl From<CreateMediumError> for ResponseError {
    fn from(err: CreateMediumError) -> Self {
        match err {
            CreateMediumError::NoDateTaken { .. } => {
                ResponseError::BadRequest {
                    message: err.to_string(),
                }
            }
            CreateMediumError::WrongAlbum { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            _ => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
        }
    }
}

impl From<MediumRepoError> for ResponseError {
    fn from(err: MediumRepoError) -> Self {
        match err {
            _ => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        match self {
            ResponseError::NotFound { message } => {
                (StatusCode::NOT_FOUND, Json(message)).into_response()
            }
            ResponseError::AuthenticationRequired => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            ResponseError::PermissionDenied { message } => {
                create_response(StatusCode::FORBIDDEN, &message, None)
            }
            ResponseError::BadRequest { message } => {
                create_response(StatusCode::BAD_REQUEST, &message, None)
            }
            ResponseError::AlreadyExists { message } => {
                create_response(StatusCode::CONFLICT, &message, None)
            }
            ResponseError::Internal {
                ref message,
                ref source,
            } => create_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                message,
                Some(source),
            ),
        }
    }
}

fn create_response(
    code: StatusCode,
    message: &String,
    source: Option<&Box<dyn BacktraceError>>,
) -> Response {
    if let Some(source) = source {
        if let Some(backtrace) = source.backtrace() {
            error!(
                "Internal server error: {}\n{}",
                source.to_string(),
                backtrace
            );
        } else {
            error!("Internal server error: {}", source.to_string());
        }
    }
    (code, Json(message)).into_response()
}
