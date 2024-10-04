mod auth;
mod info;
mod routes;

use crate::{server::routes::create_routes, user::model::UserClaims, Config};
use axum::Router;
use jwt_authorizer::{JwtAuthorizer, Validation};
use log::{debug, info};
use rdkafka::{producer::FutureProducer, ClientConfig};
use redis::aio::MultiplexedConnection;
use schema_registry_converter::async_impl::{
    avro::AvroEncoder, easy_avro::EasyAvroEncoder, schema_registry::SrSettings,
};
use snafu::{ResultExt, Whatever};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;

pub async fn run() -> Result<(), Whatever> {
    let config = Arc::new(Config::load().await?);
    let worker_client = redis::Client::open(config.cache.redis_connection.clone())
        .whatever_context("Could not establish connection to redis")?
        .get_multiplexed_tokio_connection()
        .await
        .whatever_context("Could not establish connection to redis")?;
    let meta = Arc::new(
        meta::Service::new()
            .await
            .whatever_context("Could not create exif service")?,
    );

    let app_state = AppState {
        config,
        pool: create_pool(&config)
            .await
            .whatever_context("Could not create database pool")?,
        worker_client,
        meta,
    };
    let route = create_routes(app_state).await?;

    let address = "0.0.0.0:8080";
    let listener = TcpListener::bind(address)
        .await
        .whatever_context("Could not bind to address")?;
    info!(
        "Starting photonic server at {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, route.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .whatever_context("Serve failed")?;

    Ok(())
}

async fn create_pool(config: &Arc<Config>) -> Result<PgPool, Whatever> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await?;
    Ok(pool)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    debug!("signal received, starting graceful shutdown");
}
