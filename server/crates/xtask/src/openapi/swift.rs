use std::{collections::HashMap, fs};

use serde_yaml::{Mapping, Sequence, Value};
use snafu::{ResultExt, Whatever};

/// Transforms the canonical OpenAPI 3.1 spec into a variant that
/// swift-openapi-generator can consume.
///
/// utoipa encodes `Option<T>` as a `oneOf`/`anyOf` union of a null type
/// and the inner schema. swift-openapi-generator silently drops such
/// schemas (apple/swift-openapi-generator#817), so this pass flattens
/// null-unions with exactly one non-null member into a nullable type
/// union (`type: [<t>, null]`), which the Swift generator handles
/// correctly. The canonical spec is left untouched; this is the Swift
/// consumer-side equivalent of the 3.0.3 conversion used for the Rust
/// client.
pub fn transform_openapi_for_swift(input: &str, output: &str) -> Result<(), Whatever> {
    eprintln!("Transforming OpenAPI spec for swift-openapi-generator: {input} -> {output}");

    let yaml_content = fs::read_to_string(input).whatever_context("Failed to read OpenAPI spec")?;
    let mut spec: Value =
        serde_yaml::from_str(&yaml_content).whatever_context("Failed to parse YAML")?;

    let components = extract_schema_lookup(&spec);
    sanitize_null_unions(&mut spec, &components);

    let out = serde_yaml::to_string(&spec).whatever_context("Failed to serialize YAML")?;
    fs::write(output, out).whatever_context("Failed to write transformed spec")?;

    eprintln!("✓ Swift-compatible OpenAPI spec written to {output}");
    Ok(())
}

/// Collects `components.schemas` by name so `$ref` members of null-unions
/// can be inlined during the transform.
fn extract_schema_lookup(spec: &Value) -> HashMap<String, Value> {
    let mut lookup = HashMap::new();
    if let Some(schemas) = spec
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.as_mapping())
    {
        for (name, schema) in schemas {
            if let (Some(name), Some(schema)) = (name.as_str(), schema.as_mapping()) {
                lookup.insert(name.to_string(), Value::Mapping(schema.clone()));
            }
        }
    }
    lookup
}

fn sanitize_null_unions(value: &mut Value, schemas: &HashMap<String, Value>) {
    match value {
        Value::Mapping(map) => {
            for key in ["oneOf", "anyOf"] {
                let union_key = Value::String(key.to_string());
                if let Some(Value::Sequence(entries)) = map.get(&union_key).cloned() {
                    let null_count = entries.iter().filter(|e| is_null_schema(e)).count();
                    let others: Vec<&Value> =
                        entries.iter().filter(|e| !is_null_schema(e)).collect();

                    if null_count > 0 && others.len() == 1 {
                        if let Some(flattened) = flatten_member(others[0], schemas) {
                            map.remove(&union_key);
                            for (member_key, member_value) in flattened {
                                map.entry(member_key).or_insert(member_value);
                            }
                        }
                    }
                }
            }

            for (_, v) in map.iter_mut() {
                sanitize_null_unions(v, schemas);
            }
        }
        Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                sanitize_null_unions(v, schemas);
            }
        }
        _ => {}
    }
}

fn is_null_schema(entry: &Value) -> bool {
    entry.get("type").and_then(|t| t.as_str()) == Some("null")
}

/// Flattens a non-null union member into a schema mapping with nullability
/// added to its `type`. `$ref` members are resolved one level from
/// `components.schemas`.
fn flatten_member(member: &Value, schemas: &HashMap<String, Value>) -> Option<Mapping> {
    let mut schema: Mapping = if let Some(reference) = member.get("$ref").and_then(|r| r.as_str()) {
        let name = reference.rsplit('/').next()?;
        let mut resolved = schemas.get(name)?.as_mapping()?.clone();
        // Preserve the reference's description if the schema has none.
        if let Some(description) = member.get("description") {
            resolved
                .entry(Value::String("description".to_string()))
                .or_insert(description.clone());
        }
        resolved
    } else {
        member.as_mapping()?.clone()
    };

    match schema.get(Value::String("type".to_string())) {
        Some(Value::String(t)) => {
            let types = Value::Sequence(Sequence::from(vec![
                Value::String(t.clone()),
                Value::String("null".to_string()),
            ]));
            schema.insert(Value::String("type".to_string()), types);
        }
        Some(Value::Sequence(types)) => {
            let mut types = types.clone();
            if !types.contains(&Value::String("null".to_string())) {
                types.push(Value::String("null".to_string()));
            }
            schema.insert(Value::String("type".to_string()), Value::Sequence(types));
        }
        _ => return None,
    }

    Some(schema)
}
