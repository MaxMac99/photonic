use std::{error::Error, fmt::Debug};

use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use snafu::{AsErrorSource, ErrorCompat, Snafu, Whatever};
use tracing::log::error;

pub type Result<T, E = ResponseError> = std::result::Result<T, E>;

pub(crate) trait BacktraceError: ErrorCompat + Error + AsErrorSource + Debug {}

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
    #[snafu(display("{message}"))]
    OutOfStorage { message: String },
    #[snafu(display("Internal error"))]
    Internal {
        message: String,
        source: Box<dyn BacktraceError>,
    },
}

impl From<photonic::error::Error> for ResponseError {
    fn from(err: photonic::error::Error) -> Self {
        match err {
            photonic::error::Error::Database { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            photonic::error::Error::Deadpool { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            photonic::error::Error::Interact { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            photonic::error::Error::Io { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            photonic::error::Error::Metadata { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            photonic::error::Error::FindMediumById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            photonic::error::Error::FindMediumItemById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            photonic::error::Error::FindSidecarById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            photonic::error::Error::FindUserById { .. } => ResponseError::AuthenticationRequired,
            photonic::error::Error::NoQuotaLeft { .. } => ResponseError::OutOfStorage {
                message: err.to_string(),
            },
            photonic::error::Error::FindAlbumById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            photonic::error::Error::FileAlreadyExists { .. } => ResponseError::AlreadyExists {
                message: err.to_string(),
            },
            photonic::error::Error::OutsideBaseStorage { .. } => ResponseError::BadRequest {
                message: err.to_string(),
            },
            photonic::error::Error::NoFileExtension { .. } => ResponseError::BadRequest {
                message: err.to_string(),
            },
            photonic::error::Error::NoDateTaken { .. } => ResponseError::BadRequest {
                message: err.to_string(),
            },
        }
    }
}

impl From<Whatever> for ResponseError {
    fn from(err: Whatever) -> Self {
        ResponseError::Internal {
            message: "".to_string(),
            source: Box::new(err),
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        match self {
            ResponseError::NotFound { message } => {
                (StatusCode::NOT_FOUND, Json(message)).into_response()
            }
            ResponseError::AuthenticationRequired => StatusCode::UNAUTHORIZED.into_response(),
            ResponseError::PermissionDenied { message } => {
                create_response(StatusCode::FORBIDDEN, &message, None)
            }
            ResponseError::BadRequest { message } => {
                create_response(StatusCode::BAD_REQUEST, &message, None)
            }
            ResponseError::AlreadyExists { message } => {
                create_response(StatusCode::CONFLICT, &message, None)
            }
            ResponseError::OutOfStorage { message } => {
                create_response(StatusCode::PAYLOAD_TOO_LARGE, &message, None)
            }
            ResponseError::Internal {
                ref message,
                ref source,
            } => create_response(StatusCode::INTERNAL_SERVER_ERROR, message, Some(source)),
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
            error!("{}: {}\n{}", code, source.to_string(), backtrace);
        } else {
            error!("{}: {}", code, source.to_string());
        }
    }
    (code, Json(message)).into_response()
}
