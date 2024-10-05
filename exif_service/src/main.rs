use crate::{config::ExifWorkerConfig, exiftool::Exiftool, file_handler::handle_file_created};
use common::{
    medium_item::MediumItemCreatedEvent,
    stream::{
        consumer::KafkaConsumer, events::Topic, producer::KafkaProducer, schema::register_schemata,
    },
};
use snafu::{ResultExt, Whatever};
use std::sync::Arc;
use tracing::log::{debug, error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

    info!("Starting Exif API");
    start_consumer(exiftool, consumer, producer, config).await;

    Ok(())
}

async fn start_consumer(
    exiftool: Arc<Exiftool>,
    consumer: KafkaConsumer,
    producer: KafkaProducer,
    config: Arc<ExifWorkerConfig>,
) {
    info!("Start FileCreated consumer");
    if let Some(err) = consumer
        .stream(|message: MediumItemCreatedEvent| {
            let exiftool = exiftool.clone();
            let producer = producer.clone();
            let config = config.clone();
            async move {
                debug!("Handle file created message: {:?}", message);
                if let Err(err) = handle_file_created(exiftool, producer, message, config).await {
                    error!("Error handling file created event: {}", err);
                }
                Ok(())
            }
        })
        .await
        .err()
    {
        error!("Consumer stopped unexpectedly: {}", err);
    }
}
