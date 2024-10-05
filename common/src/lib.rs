#![feature(fn_traits)]
#![feature(assert_matches)]
#![feature(box_into_inner)]
extern crate core;

pub mod config;
pub mod db;
mod domain;
pub mod error;
pub mod ksqldb;

pub mod server;
pub mod stream;

pub use domain::*;
