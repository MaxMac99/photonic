use std::{collections::HashMap, fmt::Debug};

use futures::{TryFuture, TryStreamExt};
use futures_util::TryFutureExt;
use rdkafka::{
    consumer::{CommitMode, Consumer as RdKafkaConsumer, StreamConsumer},
    message::BorrowedMessage,
    ClientConfig, Message,
};
use schema_registry_converter::async_impl::{
    easy_avro::EasyAvroDecoder, schema_registry::SrSettings,
};
use serde::Deserialize;

use crate::{
    config::KafkaConfig,
    error::{Error, Result},
    event::Event,
};

pub struct Consumer<E: Event> {
    consumer: StreamConsumer,
    decoder: EasyAvroDecoder,
}

impl<E: Event> Consumer<E> {
    pub fn new(
        config: &KafkaConfig,
        group_id: String,
        additional_opts: &HashMap<String, String>,
    ) -> Result<Self> {
        let mut client_config = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", config.broker_url.clone())
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("allow.auto.create.topics", "true");
        additional_opts
            .iter()
            .for_each(|(key, value)| client_config = client_config.set(key, value));
        let consumer: StreamConsumer = client_config.create()?;
        let sr_settings = SrSettings::new(config.schema_registry_url.clone());
        let decoder = EasyAvroDecoder::new(sr_settings);
        let topics: &[&str] = &[E::topic()];
        consumer.subscribe(topics)?;
        Ok(Consumer { consumer, decoder })
    }

    pub async fn stream<T, F, Fut>(&self, f: F) -> Result<()>
    where
        T: Clone + Debug + for<'a> Deserialize<'a> + Event,
        F: FnMut(T) -> Fut + Copy,
        Fut: TryFuture<Ok = (), Error = Error>,
    {
        self.consumer
            .stream()
            .err_into::<Error>()
            .try_for_each(|message| async move {
                self.decode(&message).and_then(f).await?;
                self.consumer
                    .commit_message(&message, CommitMode::Async)
                    .map_err(Error::from)
            })
            .await
    }

    pub async fn decode<T>(&self, message: &BorrowedMessage<'_>) -> Result<T>
    where
        T: Clone + Debug + for<'a> Deserialize<'a> + Event,
    {
        let value = self.decoder.decode(message.payload()).await?.value;
        let mut value = from_value::<T>(&value)?;
        let key = message.key().expect("Received message without key");
        value
            .store_id(&String::from_utf8(Vec::from(key)).expect("Could not convert key to string"));
        Ok(value)
    }
}
