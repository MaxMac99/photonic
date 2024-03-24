#![feature(error_generic_member_access)]
#![feature(slice_take)]

pub use uuid::Uuid;

pub use config::Config;
pub use service::Service;

pub mod config;
pub mod error;
pub mod model;
mod repository;
mod schema;
pub mod service;
mod store;
