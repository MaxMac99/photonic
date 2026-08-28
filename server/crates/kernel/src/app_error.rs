use std::backtrace::Backtrace;

use snafu::Snafu;

use crate::error::{format_snafu_error, DomainError};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum ApplicationError {
    #[snafu(transparent)]
    Domain { source: DomainError },

    #[snafu(display("Repository operation failed: {message}"))]
    Repository {
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display("External service error: {message}"))]
    ExternalService {
        message: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Internal error: {message}"))]
    Internal { message: String },

    #[snafu(display("Concurrency conflict: {message}"))]
    Conflict { message: String },
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Helper to format an application error with a readable backtrace for logging
///
/// Usage in logs: `error!(error = %format_error_with_backtrace(&e), ...)`
pub fn format_error_with_backtrace(error: &ApplicationError) -> String {
    match error {
        ApplicationError::Domain { source } => {
            // Delegate to domain error formatter
            crate::error::format_domain_error_with_backtrace(source)
        }
        ApplicationError::Internal { message } | ApplicationError::Conflict { message } => {
            message.clone()
        }
        _ => format_snafu_error(error),
    }
}
