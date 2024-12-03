use crate::{encoder::Encoder, error::Result, event::Event};
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use serde::Serialize;
use snafu::Whatever;

pub struct Producer {
    producer: kafka::producer::Producer,
    encoder: Encoder,
}

impl Producer {
    pub fn new(
        producer: kafka::producer::Producer,
        schema_registry_url: String,
    ) -> std::result::Result<Self, Whatever> {
        let sr_settings = SrSettings::new(schema_registry_url);
        let encoder = Encoder::new(sr_settings);
        Ok(Self { producer, encoder })
    }

    pub async fn send<E, K>(&self, event: E) -> Result<()>
    where
        E: Serialize + Event<K>,
    {
        let record = self.encoder.encode_event(event).await?;
        self.producer.send(&record)?;
        Ok(())
    }

    pub async fn tombstone_event<T>(&self, event: T) -> Result<()>
    where
        T: Event,
    {
        let topic = &T::topic().subject_name();
        let key = event.id();
        let record = FutureRecord::to(&topic).payload("").key(&key);
        self.publish(record).await
    }

    pub async fn tombstone(&self, topic: Topic, key: String) -> Result<()> {
        let topic = topic.to_string();
        let record = FutureRecord::to(&topic).payload("").key(&key);
        self.publish(record).await
    }
}
