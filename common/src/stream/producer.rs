use std::{sync::Arc, time::Duration};

use rdkafka::{
    message::ToBytes,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use schema_registry_converter::{
    async_impl::{easy_avro::EasyAvroEncoder, schema_registry::SrSettings},
    schema_registry_common::SubjectNameStrategy,
};
use serde::Serialize;
use snafu::{ResultExt, Whatever};
use tracing::log::debug;

use crate::{
    config::StreamConfig,
    error::{Error, Result},
    stream::events::{Event, Topic},
};

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    encoder: Arc<EasyAvroEncoder>,
}

impl KafkaProducer {
    pub fn new(config: &StreamConfig) -> std::result::Result<Self, Whatever> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", config.broker_url.clone())
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.messages", "10")
            .create()
            .whatever_context("Could not create producer")?;
        let sr_settings = SrSettings::new(config.schema_registry_url.clone());
        let encoder = EasyAvroEncoder::new(sr_settings);
        Ok(Self {
            producer,
            encoder: Arc::new(encoder),
        })
    }

    pub async fn produce<T>(&self, event: T) -> Result<()>
    where
        T: Serialize + Event,
    {
        let topic = T::topic().subject_name();
        let strategy = SubjectNameStrategy::TopicNameStrategy(topic.clone(), false);
        let key = event.id();
        let payload = self.encoder.encode_struct(event, &strategy).await?;

        let record = FutureRecord::to(&topic).payload(&payload).key(&key);
        debug!("Published message with key {} to topic {}", key, topic);
        self.publish(record).await
    }

    pub async fn tombstone_event<T>(&self, event: T) -> Result<()>
    where
        T: Event,
    {
        let topic = &T::topic().subject_name();
        let key = event.id();
        let record = FutureRecord::to(&topic).payload("").key(&key);
        debug!("Published tombstone with key {} to topic {}", key, topic);
        self.publish(record).await
    }

    pub async fn tombstone(&self, topic: Topic, key: String) -> Result<()> {
        let topic = topic.to_string();
        let record = FutureRecord::to(&topic).payload("").key(&key);
        debug!("Published tombstone with key {} to topic {}", key, topic);
        self.publish(record).await
    }

    async fn publish<K, P>(&self, record: FutureRecord<'_, K, P>) -> Result<()>
    where
        K: ToBytes + ?Sized,
        P: ToBytes + ?Sized,
    {
        let status = self.producer.send(record, Duration::from_secs(2)).await;
        if let Some(err) = status.err() {
            return Err(Error::from(err.0));
        }
        Ok(())
    }
}
