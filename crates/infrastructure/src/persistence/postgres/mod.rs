pub mod checkpoint_store;
pub mod es_snapshot_store;
pub mod events;
mod groups;
pub mod medium;
pub mod metadata;
pub mod snapshot_store;
pub mod stream_link_store;
pub mod task;
pub mod transaction_provider;
pub mod user;

use std::backtrace::Backtrace;

use domain::error::DomainError;

pub(crate) fn repo_error(e: sqlx::Error) -> DomainError {
    DomainError::Repository {
        message: e.to_string(),
        backtrace: Backtrace::capture(),
    }
}
