use crate::{encoder::Encoder, error::Result, kafka::producer::AsBytes};
use avro_reference::{AvroReferenceSchema, ReferenceSchema};
use schema_registry_converter::{
    avro_common::get_supplied_schema,
    schema_registry_common::{SuppliedReference, SuppliedSchema},
};

pub enum KeyType {
    None,
    String,
    Avro,
}

pub trait EventKey {
    async fn encode(&self, encoder: &Encoder, topic: String) -> Result<Vec<u8>>;
}

impl<T> EventKey for T
where
    T: AsBytes,
{
    async fn encode(&self, _: &Encoder, _: String) -> Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}

pub trait Event<K: EventKey>: AvroReferenceSchema {
    fn topic() -> &'static str;
    fn key(&self) -> K;
    fn partition(&self) -> Option<i32> {
        None
    }
}

pub fn create_supplied_schema_with_references<T: AvroReferenceSchema>() -> SuppliedSchema {
    let schema = T::get_reference_schema();
    let mut supplied = get_supplied_schema(&schema.schema);
    supplied.references = create_supplied_references(schema.references);
    supplied
}

fn create_supplied_references(references: Vec<ReferenceSchema>) -> Vec<SuppliedReference> {
    references
        .into_iter()
        .map(|reference| SuppliedReference {
            name: reference.name.to_string(),
            subject: reference.name.to_string(),
            schema: reference.schema.canonical_form(),
            references: create_supplied_references(reference.references),
        })
        .collect()
}
