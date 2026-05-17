// DDD/event-sourcing layout: `medium/medium.rs` etc. name the aggregate root
// after its bounded context, and event/value-object constructors legitimately
// take many fields.
#![allow(clippy::module_inception, clippy::too_many_arguments)]

pub mod aggregate;
// pub mod album;
pub mod error;
pub mod event;
pub mod medium;
pub mod metadata;
pub mod serde_helpers;
pub mod shared;
pub mod task;
pub mod user;

pub use serde_helpers::*;
