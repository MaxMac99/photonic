use deadpool_redis::{Pool, Runtime};
use ksqldb::KsqlDB;
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use sqlx::PgPool;
use std::sync::Arc;
pub use uuid::Uuid;

use crate::common::Producer;
pub use config::Config;

mod album;
mod common;
mod config;
mod db;
pub mod error;
mod exif;
mod medium;
mod medium_item;
mod server;
pub mod service;
mod sidecar;
mod store;
mod user;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db_pool: PgPool,
    pub cache_pool: Pool,
    pub producer: Producer,
    pub ksql_db: Arc<KsqlDB>,
    pub meta: Arc<exif::Service>,
}

impl AppState {
    async fn new(config: Arc<Config>) -> Result<Self, Whatever> {
        let pool = db::init_db(config.clone()).await?;
        let cache_pool =
            deadpool_redis::Config::from_connection_info(&config.cache.redis_connection)
                .create_pool(Some(Runtime::Tokio1))
                .whatever_context("Could not create cache pool")?;
        let producer = Producer::new(config.clone());
        let ksql_db = KsqlDB::new(config.streams.ksql_db_url.clone(), Client::builder(), false)
            .whatever_context("Could not create ksql db context")?;

        Self {
            config,
            db_pool: pool,
            cache_pool,
            producer,
            ksql_db: Arc::new(ksql_db),
            meta: Arc::new(()),
        }
    }
}
