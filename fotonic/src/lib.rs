#![feature(error_generic_member_access)]

pub use mongodb::bson::oid::ObjectId;

pub use config::Config;
pub use service::Service;

pub mod config;
pub mod model;
mod repository;
pub mod service;
mod store;
