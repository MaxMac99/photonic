use crate::{
    error::Result,
    event::{create_supplied_schema_with_references, Event, EventKey},
    kafka::producer::Record,
};
use avro_reference::AvroReferenceSchema;
use schema_registry_converter::{
    async_impl::{easy_avro::EasyAvroEncoder, schema_registry::SrSettings},
    avro_common::get_supplied_schema,
    schema_registry_common::SubjectNameStrategy,
};
use serde::Serialize;

pub struct Encoder {
    encoder: EasyAvroEncoder,
}

impl Encoder {
    pub fn new(sr_settings: SrSettings) -> Self {
        Self {
            encoder: EasyAvroEncoder::new(sr_settings),
        }
    }

    pub(crate) async fn encode_event<'a, K, E>(&self, event: E) -> Result<Record<Vec<u8>, Vec<u8>>>
    where
        K: EventKey,
        E: Event<K> + Serialize,
    {
        let key = event.key().encode(&self, E::topic().into()).await?;
        let schema = create_supplied_schema_with_references::<E>();
        let strategy =
            SubjectNameStrategy::TopicNameStrategyWithSchema(E::topic().into(), false, schema);
        let partition = event.partition();
        let payload = self.encoder.encode_struct(event, &strategy).await?;

        let mut record = Record::from_key_value(E::topic(), key, payload);
        if let Some(partition) = partition {
            record = record.with_partition(partition);
        }
        Ok(record)
    }

    pub(crate) async fn encode_key<T>(&self, key: T, topic: String) -> Result<Vec<u8>>
    where
        T: EventKey + AvroReferenceSchema + Serialize,
    {
        let schema = T::get_reference_schema().schema;
        let supplied_schema = get_supplied_schema(&schema);
        let strategy =
            SubjectNameStrategy::TopicNameStrategyWithSchema(topic, true, supplied_schema);
        Ok(self.encoder.encode_struct(key, &strategy).await?)
    }
}
