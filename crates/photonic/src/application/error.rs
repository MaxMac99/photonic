use std::backtrace::Backtrace;

use snafu::{ErrorCompat, Snafu};

use crate::domain::error::{format_error_with_backtrace as format_domain_error, DomainError};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
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
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Helper to format an application error with a readable backtrace for logging
///
/// Usage in logs: `error!(error = %format_error_with_backtrace(&e), ...)`
pub fn format_error_with_backtrace(error: &ApplicationError) -> String {
    match error {
        ApplicationError::Domain { source } => {
            // Delegate to domain error formatter
            format_domain_error(source)
        }
        _ => {
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
    }
}
