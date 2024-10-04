use std::sync::Arc;

use axum::Router;
use snafu::{ResultExt, Whatever};
use tokio::{
    net::TcpListener,
    sync::{oneshot, oneshot::Sender},
};
use tracing::log::{debug, error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::ExifWorkerConfig, exiftool::Exiftool, file_handler::handle_file_created};
use common::{
    server::shutdown_signal_with_external_signal,
    stream::{
        consumer::KafkaConsumer,
        events::{MediumItemCreatedEvent, Topic},
        producer::KafkaProducer,
        schema::register_schemata,
    },
};

mod config;
mod exiftool;
mod file_handler;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let config = Arc::new(ExifWorkerConfig::load().await?);
    let exiftool = Arc::new(
        Exiftool::new()
            .await
            .whatever_context("Could not create exiftool")?,
    );

    let app = Router::new();

    let listener = TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await
        .whatever_context("Could not bind to address")?;

    register_schemata(
        &config.stream,
        vec![Topic::MediumItemCreated, Topic::MediumItemExifLoaded],
    )
    .await?;

    let producer =
        KafkaProducer::new(&config.stream).whatever_context("Could not create producer")?;
    let consumer = KafkaConsumer::new(
        &config.stream,
        "exif".to_string(),
        &vec![Topic::MediumItemCreated],
    )
    .whatever_context("Could not create consumer")?;

    let (died_sender, died_receiver) = oneshot::channel();
    tokio::spawn(start_consumer(
        exiftool,
        consumer,
        producer,
        config,
        died_sender,
    ));

    info!("Starting Exif API");
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal_with_external_signal(died_receiver))
        .await
        .whatever_context("Could not start server")?;

    Ok(())
}

async fn start_consumer(
    exiftool: Arc<Exiftool>,
    consumer: KafkaConsumer,
    producer: KafkaProducer,
    config: Arc<ExifWorkerConfig>,
    died_sender: Sender<bool>,
) {
    info!("Start FileCreated consumer");
    if let Some(err) = consumer
        .stream(|message: MediumItemCreatedEvent| {
            let exiftool = exiftool.clone();
            let producer = producer.clone();
            let config = config.clone();
            async move {
                debug!("Handle file created message: {:?}", message);
                let _ = handle_file_created(exiftool, producer, message, config).await?;
                Ok(())
            }
        })
        .await
        .err()
    {
        error!("Consumer stopped unexpectedly: {}", err);
    }
    died_sender.send(true).unwrap();
}
