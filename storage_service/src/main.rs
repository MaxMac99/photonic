use crate::{config::StorageWorkerConfig, state::AppState};
use common::{
    db::init_db,
    server::shutdown_signal,
    stream::{events::Topic, producer::KafkaProducer, schema::register_schemata},
};
use snafu::{ResultExt, Whatever};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::log::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

    let config = Arc::new(StorageWorkerConfig::load().await?);
    register_schemata(
        &config.stream,
        vec![Topic::MediumItemCreated, Topic::MediumItemExifLoaded],
    )
    .await?;
    let producer = KafkaProducer::new(&config.stream)?;
    let db_pool = init_db(&config.clone().database, run_migrations).await?;

    let state = AppState {
        config: config.clone(),
        producer,
        db_pool,
    };

    let app = create_app(&config, state.clone()).await?;

    let listener = TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await
        .whatever_context("Could not bind to address")?;

    info!("Starting storage API");
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .whatever_context("Could not start server")?;

    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<(), Whatever> {
    sqlx::migrate!()
        .run(pool)
        .await
        .whatever_context("Could not run migrations")
}
