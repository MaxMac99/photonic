use serde::Deserialize;
use serde_json::Value;
use snafu::Snafu;
use std::{backtrace::Backtrace, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(transparent)]
    Http {
        source: reqwest::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Json {
        source: serde_json::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Error while processing KSQL DB stream: {message}"))]
    Stream {
        message: String,
        backtrace: Backtrace,
    },
    #[snafu(display("Received final message before closing stream: {message}"))]
    FinalMessage {
        message: String,
        backtrace: Backtrace,
    },
    #[snafu(display("{source}"))]
    KsqlDb {
        #[snafu(source(false))]
        source: KsqlDbError,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct KsqlDbError {
    #[serde(rename = "@type")]
    pub response_type: String,
    pub statement_text: Option<String>,
    #[serde(rename = "error_code")]
    pub error_code: Option<u32>,
    pub message: Option<String>,
    pub entities: Option<Vec<Value>>,
}

impl fmt::Display for KsqlDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Received error from KSQL DB: response type '{}', with error code [{}] and message: '{}'",
            self.response_type,
            self.error_code.unwrap_or_default(),
            self.message.clone().unwrap_or_default()
        )
    }
}
