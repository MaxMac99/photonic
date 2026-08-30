//! Composition root: assembles the application from modules and adapters.
//!
//! This crate is the only place that knows all concrete types. It wires
//! adapters to module ports, registers listeners and background tasks, and
//! owns configuration and the server runtime.

#![allow(clippy::new_without_default)]
// derive-new 0.7 generates `Field { field: field }` constructors, which
// newer clippy flags as redundant field names. Allow at crate root until
// derive-new emits field shorthand.
#![allow(clippy::redundant_field_names)]

pub mod config;
pub mod db;
pub mod di;
pub mod events;
pub mod listeners;
pub mod server;
pub mod tasks;

pub use config::{DatabaseConfig, GlobalConfig, ServerConfig, StorageConfig};
pub use di::Container;
pub use server::{run_server, setup_test_tracing, setup_tracing, ServerHandle};
