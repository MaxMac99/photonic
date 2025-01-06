use crate::storage::StorageVariant;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use futures_util::task::SpawnError;
use snafu::{ErrorCompat, Snafu};
use std::{backtrace::Backtrace, fmt::Debug, path::PathBuf};
use tracing::log::error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(transparent)]
    Io {
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Database {
        source: sqlx::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Spawn {
        source: SpawnError,
        backtrace: Backtrace,
    },
    #[snafu(display("The function is currently under development and not implemented yet"))]
    NotImplemented { backtrace: Backtrace },
    #[snafu(display("Could not parse: {message}"))]
    Parse {
        message: String,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not publish event to topic {topic}: {send_error}"))]
    PublishEvent {
        topic: String,
        backtrace: Backtrace,
        send_error: String,
    },
    #[snafu(display("The path {path:?} does not exists"))]
    FileNotExists { path: PathBuf, backtrace: Backtrace },
    #[snafu(display("The path {path:?} is not valid"))]
    InvalidPath { path: PathBuf, backtrace: Backtrace },
    #[snafu(display("The quota was exceeded"))]
    QuotaExceeded { backtrace: Backtrace },
    #[snafu(display("The medium with id {} was not found", id))]
    MediumNotFound { id: Uuid, backtrace: Backtrace },
    #[snafu(display(
        "The storage variant {variant} for medium item with id {medium_item_id} was not found"
    ))]
    StorageVariantNotFound {
        medium_item_id: Uuid,
        variant: StorageVariant,
        backtrace: Backtrace,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::MediumNotFound { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if let Some(backtrace) = self.backtrace() {
            error!("{}: {}\n{}", status, self, backtrace);
        } else {
            error!("{}: {}", status, self);
        }
        (status, Json(self.to_string())).into_response()
    }
}
