use crate::{config::StreamConfig, stream::events::Topic};
use avro_reference::ReferenceSchema;
use schema_registry_converter::{
    async_impl::schema_registry::{get_schema_by_subject, SrSettings},
    avro_common::get_supplied_schema,
    schema_registry_common::{SubjectNameStrategy, SuppliedReference},
};
use snafu::{ResultExt, Whatever};

pub trait AvroSchemaExt {}

pub async fn register_schemata(
    config: &StreamConfig,
    event_schemata: Vec<Topic>,
) -> Result<(), Whatever> {
    let sr_settings = SrSettings::new(config.schema_registry_url.clone());
    let strategies = create_supplied_schemata_with_references(event_schemata);
    for strategy in strategies {
        get_schema_by_subject(&sr_settings, &strategy)
            .await
            .whatever_context("Could not register schema")?;
    }
    Ok(())
}

fn create_supplied_schemata_with_references(
    event_schemata: Vec<Topic>,
) -> Vec<SubjectNameStrategy> {
    event_schemata
        .iter()
        .map(|topic| {
            let schema = topic.schema();
            let mut supplied = get_supplied_schema(&schema.schema);
            supplied.references = create_reference(schema.references);
            SubjectNameStrategy::TopicNameStrategyWithSchema(topic.subject_name(), false, supplied)
        })
        .collect()
}

fn create_reference(reference_schemata: Vec<ReferenceSchema>) -> Vec<SuppliedReference> {
    reference_schemata
        .into_iter()
        .map(|reference_schema| {
            let name = reference_schema.name.fully_qualified_name(&None);
            SuppliedReference {
                name: name.to_string(),
                subject: name.to_string(),
                schema: reference_schema.schema.canonical_form(),
                references: create_reference(reference_schema.references),
            }
        })
        .collect()
}
