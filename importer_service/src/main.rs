extern crate core;

use crate::{api::create_app, config::ImporterWorkerConfig, state::AppState};
use common::{
    ksqldb::KsqlDb,
    server::shutdown_signal,
    stream::{events::Topic, producer::KafkaProducer, schema::register_schemata},
    user::setup_streams_and_tables,
};
use snafu::{ResultExt, Whatever};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::log::info;
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

mod api;
mod config;
mod medium;
mod medium_item;
mod state;
mod storage;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let config = Arc::new(ImporterWorkerConfig::load().await?);
    register_schemata(
        &config.stream,
        vec![Topic::MediumItemCreated, Topic::MediumItemExifLoaded],
    )
    .await?;
    let producer = KafkaProducer::new(&config.stream)?;
    let ksql_db = Arc::new(KsqlDb::new(config.stream.ksqldb_url.clone()));
    setup_streams_and_tables(ksql_db.clone()).await?;

    let state = AppState {
        config: config.clone(),
        producer,
        ksql_db,
    };

    let app = create_app(&config, state.clone()).await?;

    let listener = TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await
        .whatever_context("Could not bind to address")?;

    info!("Starting importer API");
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .whatever_context("Could not start server")?;

    Ok(())
}
