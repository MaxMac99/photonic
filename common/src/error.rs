use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rdkafka::error::KafkaError;
use schema_registry_converter::error::SRCError;
use snafu::{ErrorCompat, Snafu};
use std::{backtrace::Backtrace, fmt::Debug, path::PathBuf};
use tracing::log::error;

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
    SchemaRegistry {
        source: SRCError,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Kafka {
        source: KafkaError,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    AvroDecode {
        source: apache_avro::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Database {
        source: sqlx::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    KsqlDb {
        source: crate::ksqldb::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("The function is currently under development and not implemented yet"))]
    NotImplemented { backtrace: Backtrace },
    #[snafu(display("Could not parse response: {message}"))]
    Parse {
        message: String,
        backtrace: Backtrace,
    },
    #[snafu(display("The path {path:?} does not exists"))]
    FileNotExists { path: PathBuf, backtrace: Backtrace },
    #[snafu(display("The path {path:?} is not valid"))]
    InvalidPath { path: PathBuf, backtrace: Backtrace },
    #[snafu(display("The quota was exceeded"))]
    QuotaExceeded { backtrace: Backtrace },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
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
