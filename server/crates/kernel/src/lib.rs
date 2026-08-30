// derive-new 0.7 generates `Field { field: field }` constructors, which
// newer clippy flags as redundant field names. Allow at crate root until
// derive-new emits field shorthand.
#![allow(clippy::redundant_field_names)]

pub mod aggregate;
pub mod app_error;
pub mod error;
pub mod event;
pub mod event_bus;
pub mod file;
pub mod ids;
pub mod serde_helpers;
pub mod shared;
pub mod storage;

pub use aggregate::{AggregateRoot, AggregateVersion};
pub use app_error::{ApplicationError, ApplicationResult};
pub use error::{DomainError, DomainResult};
pub use event::{DomainEvent, EventMetadata};
pub use event_bus::PublishEvent;
pub use file::*;
pub use ids::*;
pub use serde_helpers::*;
pub use storage::{FileLocation, FileMetadata, StorageTier};
