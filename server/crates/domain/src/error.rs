use std::{backtrace::Backtrace, path::PathBuf};

use byte_unit::Byte;
use snafu::{ErrorCompat, Snafu};
use uuid::Uuid;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum DomainError {
    #[snafu(display("Validation error: {message}"))]
    Validation {
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Quota exceeded: required {required}, available {available}"))]
    QuotaExceeded {
        required: Byte,
        available: Byte,
        backtrace: Backtrace,
    },

    #[snafu(display("Invariant violation: {message}"))]
    InvariantViolation {
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Concurrent modification detected for aggregate {aggregate_id} (expected version {expected_version})"
    ))]
    ConcurrentModification {
        aggregate_id: Uuid,
        expected_version: i64,
        backtrace: Backtrace,
    },

    #[snafu(display("{entity} not found: {id}"))]
    EntityNotFound {
        entity: &'static str,
        id: Uuid,
        backtrace: Backtrace,
    },

    #[snafu(display("Could not parse: {message}"))]
    Parse {
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display("The path {path:?} does not exist"))]
    FileNotExists { path: PathBuf, backtrace: Backtrace },

    #[snafu(display("The path {path:?} is not valid"))]
    InvalidPath { path: PathBuf, backtrace: Backtrace },

    #[snafu(display("Repository error: {message}"))]
    Repository {
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Storage error: {message}"))]
    Storage {
        message: String,
        backtrace: Backtrace,
    },
}

impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        DomainError::Storage {
            message: err.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

pub type DomainResult<T> = Result<T, DomainError>;

/// Helper to format an error with a readable backtrace for logging
///
/// Usage in logs: `error!(error = %format_error_with_backtrace(&e), ...)`
pub fn format_error_with_backtrace(error: &DomainError) -> String {
    let mut output = error.to_string();

    if let Some(backtrace) = ErrorCompat::backtrace(error) {
        let backtrace_str = backtrace.to_string();
        if !backtrace_str.trim().is_empty() && backtrace_str != "disabled backtrace" {
            output.push_str("\n\nBacktrace:\n");
            output.push_str(&backtrace_str);
        }
    }

    output
}
