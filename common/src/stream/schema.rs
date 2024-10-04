use crate::{
    config::StreamConfig,
    stream::events::{Topic, COMMON_SCHEMATA},
};
use apache_avro::{
    schema::{EnumSchema, FixedSchema, Name, Namespace, RecordSchema},
    Schema,
};
use schema_registry_converter::{
    async_impl::schema_registry::{get_schema_by_subject, SrSettings},
    avro_common::get_supplied_schema,
    schema_registry_common::{SubjectNameStrategy, SuppliedReference},
};
use snafu::{whatever, ResultExt, Whatever};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait AvroSchemaExt {}

pub async fn register_schemata(
    config: &StreamConfig,
    event_schemata: Vec<Topic>,
) -> Result<(), Whatever> {
    let sr_settings = SrSettings::new(config.schema_registry_url.clone());
    let strategies =
        create_supplied_schemata_with_references(event_schemata, (*COMMON_SCHEMATA).to_vec())?;
    for strategy in strategies {
        get_schema_by_subject(&sr_settings, &strategy)
            .await
            .whatever_context("Could not register schema")?;
    }
    Ok(())
}

fn create_supplied_schemata_with_references(
    event_schemata: Vec<Topic>,
    common_schemata: Vec<&Schema>,
) -> Result<Vec<SubjectNameStrategy>, Whatever> {
    let mut names = HashMap::new();
    let all_schemata = event_schemata
        .iter()
        .map(|topic| (topic.subject_name(), topic.schema()))
        .chain(
            common_schemata
                .iter()
                .map(|&schema| (get_full_qualified_name(schema).unwrap(), schema)),
        )
        .collect::<Vec<(String, &Schema)>>();

    resolve(
        &mut names,
        all_schemata
            .iter()
            .map(|(name, &ref schema)| (name, schema))
            .collect(),
        None,
        &None,
    )?;

    let mut cache = HashMap::new();
    event_schemata
        .iter()
        .map(|topic| {
            let schema = topic.schema();
            let mut supplied = get_supplied_schema(schema);
            supplied.references = create_references(schema, &names, &None, &mut cache)?;
            Ok(SubjectNameStrategy::TopicNameStrategyWithSchema(
                topic.subject_name(),
                false,
                supplied,
            ))
        })
        .collect()
}

fn get_full_qualified_name(schema: &Schema) -> Option<String> {
    match schema {
        Schema::Record(schema) => match &schema.name.namespace {
            None => Some(schema.name.name.clone()),
            Some(namespace) => Some(format!("{}.{}", namespace, schema.name.name)),
        },
        _ => None,
    }
}

fn resolve<'a>(
    names: &mut HashMap<Name, (&'a String, &'a Schema)>,
    schemata: Vec<(&'a String, &'a Schema)>,
    global_schema: Option<(&'a String, &'a Schema)>,
    enclosing_namespace: &Namespace,
) -> Result<(), Whatever> {
    for (schema_name, schema) in schemata {
        match schema {
            Schema::Array(schema) | Schema::Map(schema) => resolve(
                names,
                vec![(schema_name, schema)],
                global_schema.or_else(|| Some((schema_name, schema))),
                enclosing_namespace,
            )?,
            Schema::Union(union) => {
                for schema in union.variants() {
                    resolve(
                        names,
                        vec![(schema_name, schema)],
                        global_schema
                            .clone()
                            .or_else(|| Some((schema_name, schema))),
                        enclosing_namespace,
                    )?
                }
            }
            Schema::Enum(EnumSchema { name, .. }) | Schema::Fixed(FixedSchema { name, .. }) => {
                let fully_qualified_name = name.fully_qualified_name(enclosing_namespace);
                if names
                    .insert(
                        fully_qualified_name.clone(),
                        global_schema.unwrap_or_else(|| (schema_name, schema)),
                    )
                    .is_some()
                {
                    whatever!("schema {} already exists", fully_qualified_name);
                }
            }
            Schema::Record(RecordSchema { name, fields, .. }) => {
                let fully_qualified_name = name.fully_qualified_name(enclosing_namespace);
                if names
                    .insert(
                        fully_qualified_name.clone(),
                        global_schema.unwrap_or_else(|| (schema_name, schema)),
                    )
                    .is_some()
                {
                    whatever!("schema {} already exists", fully_qualified_name);
                }

                let record_namespace = fully_qualified_name.namespace;
                for field in fields {
                    resolve(
                        names,
                        vec![(schema_name, &field.schema)],
                        global_schema.or_else(|| Some((schema_name, schema))),
                        &record_namespace,
                    )?
                }
            }
            _ => (),
        }
    }
    Ok(())
}

fn create_references(
    schema: &Schema,
    names: &HashMap<Name, (&String, &Schema)>,
    enclosing_namespace: &Namespace,
    cache: &mut HashMap<Name, Vec<Rc<RefCell<SuppliedReference>>>>,
) -> Result<Vec<SuppliedReference>, Whatever> {
    match schema {
        Schema::Array(nested_schema) | Schema::Map(nested_schema) => {
            create_references(nested_schema, names, enclosing_namespace, cache)
        }
        Schema::Union(union) => Ok(union
            .variants()
            .iter()
            .map(|schema| create_references(schema, names, enclosing_namespace, cache))
            .collect::<Result<Vec<Vec<SuppliedReference>>, Whatever>>()?
            .into_iter()
            .flatten()
            .collect()),
        Schema::Record(RecordSchema { name, fields, .. }) => {
            let fully_qualified_name = name.fully_qualified_name(enclosing_namespace);
            let record_namespace = fully_qualified_name.namespace;
            Ok(fields
                .iter()
                .map(|field| create_references(&field.schema, names, &record_namespace, cache))
                .collect::<Result<Vec<Vec<SuppliedReference>>, Whatever>>()?
                .into_iter()
                .flatten()
                .collect())
        }
        Schema::Ref { name } => {
            let fully_qualified_name = name.fully_qualified_name(enclosing_namespace);
            create_reference(names, enclosing_namespace, cache, fully_qualified_name)
        }
        _ => Ok(vec![]),
    }
}

fn create_reference(
    names: &HashMap<Name, (&String, &Schema)>,
    enclosing_namespace: &Namespace,
    cache: &mut HashMap<Name, Vec<Rc<RefCell<SuppliedReference>>>>,
    fully_qualified_name: Name,
) -> Result<Vec<SuppliedReference>, Whatever> {
    if let Some(reference) = cache.get(&fully_qualified_name) {
        Ok(reference
            .clone()
            .into_iter()
            .map(|boxed| boxed.borrow().clone())
            .collect())
    } else if let Some((name, schema)) = names.get(&fully_qualified_name) {
        let reference = Rc::new(RefCell::new(SuppliedReference {
            name: name.to_string(),
            subject: name.to_string(),
            schema: schema.canonical_form(),
            references: vec![],
        }));
        cache.insert(fully_qualified_name, vec![reference.clone()]);
        let references = create_references(schema, names, enclosing_namespace, cache)?;
        let mut reference = reference.borrow_mut();
        reference.references = references;
        Ok(vec![reference.clone()])
    } else {
        whatever!("Could not resolve reference {}", fully_qualified_name)
    }
}

#[cfg(test)]
mod tests {
    use crate::stream::{
        events::{Topic, COMMON_SCHEMATA},
        schema::create_supplied_schemata_with_references,
    };
    use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
    use std::assert_matches::assert_matches;

    #[test]
    fn test_create_supplied_schemata_with_references() {
        let strategies = create_supplied_schemata_with_references(
            vec![Topic::MediumItemExifLoaded],
            (*COMMON_SCHEMATA).to_vec(),
        )
        .unwrap();

        assert_eq!(strategies.len(), 1);
        assert_matches!(
            strategies.get(0).unwrap(),
            SubjectNameStrategy::TopicNameStrategyWithSchema(topic, false, schema) if topic == "EXIF_LOADED" && schema.references.get(0).unwrap().name == "common.DateTimeTz"
        )
    }
}
