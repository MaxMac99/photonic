use std::{error::Error, fmt::Debug};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
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

impl From<fotonic::error::Error> for ResponseError {
    fn from(err: fotonic::error::Error) -> Self {
        match err {
            fotonic::error::Error::MongoDb { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            fotonic::error::Error::Io { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            fotonic::error::Error::Metadata { .. } => ResponseError::Internal {
                message: "".to_string(),
                source: Box::new(err),
            },
            fotonic::error::Error::FindMediumById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            fotonic::error::Error::FindMediumItemById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            fotonic::error::Error::FindUserById { .. } => ResponseError::AuthenticationRequired,
            fotonic::error::Error::NoQuotaLeft { .. } => ResponseError::OutOfStorage {
                message: err.to_string(),
            },
            fotonic::error::Error::FindAlbumById { .. } => ResponseError::NotFound {
                message: err.to_string(),
            },
            fotonic::error::Error::FileAlreadyExists { .. } => ResponseError::AlreadyExists {
                message: err.to_string(),
            },
            fotonic::error::Error::OutsideBaseStorage { .. } => ResponseError::BadRequest {
                message: err.to_string(),
            },
            fotonic::error::Error::NoFileExtension { .. } => ResponseError::BadRequest {
                message: err.to_string(),
            },
            fotonic::error::Error::NoDateTaken { .. } => ResponseError::BadRequest {
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
