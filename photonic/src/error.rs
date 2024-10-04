use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::{backtrace::Backtrace, fmt::Debug};

use serde_json::to_string;
use snafu::{AsErrorSource, ErrorCompat, Snafu};
use tracing::error;
use uuid::Uuid;

use crate::medium_item::model::MediumItemType;
use meta::MetaError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(transparent)]
    Database {
        source: sqlx::error::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Io {
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Metadata {
        source: MetaError,
        backtrace: Backtrace,
    },
    // Custom errors
    #[snafu(display("Could not find medium with id {id}"))]
    FindMediumById { id: Uuid, backtrace: Backtrace },
    #[snafu(display("Could not find medium item of type {medium_type:?}"))]
    FindMediumItemById {
        medium_type: MediumItemType,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not find sidecar"))]
    FindSidecarById { backtrace: Backtrace },
    #[snafu(display("Could not find album with id {id}"))]
    FindAlbumById { id: Uuid, backtrace: Backtrace },
    #[snafu(display("Could not find user with id {id}"))]
    FindUserById { id: Uuid, backtrace: Backtrace },
    #[snafu(display("There is not enough quota left"))]
    NoQuotaLeft { backtrace: Backtrace },
    #[snafu(display("The given file is outside the base storage"))]
    OutsideBaseStorage { backtrace: Backtrace },
    #[snafu(display("Could not find a file extension"))]
    NoFileExtension { backtrace: Backtrace },
    #[snafu(display("The given file already exists"))]
    FileAlreadyExists { backtrace: Backtrace },
    #[snafu(display("Could not find the date when this medium was taken"))]
    NoDateTaken { backtrace: Backtrace },
    #[snafu(display("The function is currently under development and not implemented yet"))]
    NotImplemented { backtrace: Backtrace },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Io { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Metadata { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Error::FindMediumById { .. } => StatusCode::NOT_FOUND,
            Error::FindMediumItemById { .. } => StatusCode::NOT_FOUND,
            Error::FindSidecarById { .. } => StatusCode::NOT_FOUND,
            Error::FindAlbumById { .. } => StatusCode::NOT_FOUND,
            Error::FindUserById { .. } => StatusCode::UNAUTHORIZED,
            Error::NoQuotaLeft { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            Error::OutsideBaseStorage { .. } => StatusCode::BAD_REQUEST,
            Error::NoFileExtension { .. } => StatusCode::BAD_REQUEST,
            Error::FileAlreadyExists { .. } => StatusCode::CONFLICT,
            Error::NoDateTaken { .. } => StatusCode::BAD_REQUEST,
            Error::NotImplemented { .. } => StatusCode::NOT_IMPLEMENTED,
        };
        error!("{}: {}\n{}", status, self, self.backtrace());
        (status, Json(self.to_string())).into_response()
    }
}
