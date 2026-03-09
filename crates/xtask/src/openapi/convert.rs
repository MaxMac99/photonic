use std::{fs, process::Command};

use serde_yaml::Value;
use snafu::Whatever;

pub fn convert_openapi(input: &str, output: &str) -> Result<(), Whatever> {
    // Step 1: Convert OpenAPI 3.1.x to 3.0.3 using openapi-down-convert
    eprintln!(
        "Converting OpenAPI 3.1.x to 3.0.3 from {} to {}...",
        input, output
    );
    let convert_output = Command::new("openapi-down-convert")
        .args(["--input", input, "--output", output])
        .output()
        .expect("Failed to run openapi-down-convert");

    if !convert_output.status.success() {
        eprintln!(
            "openapi-down-convert stderr: {}",
            String::from_utf8_lossy(&convert_output.stderr)
        );
        panic!("Failed to convert OpenAPI spec to 3.0.3");
    }

    // Step 2: Post-process to remove any remaining 'null' types
    eprintln!("Post-processing to remove remaining null types...");
    let yaml_content = fs::read_to_string(output).expect("Failed to read converted spec");

    let mut spec: Value =
        serde_yaml::from_str(&yaml_content).expect("Failed to parse converted YAML");

    remove_null_types(&mut spec);
    replace_wildcard_content_types(&mut spec);
    convert_array_query_params_to_string(&mut spec);

    let fixed_yaml = serde_yaml::to_string(&spec).expect("Failed to serialize fixed YAML");

    fs::write(output, fixed_yaml).expect("Failed to write fixed spec");

    eprintln!("✓ Converted to OpenAPI 3.0.3");
    Ok(())
}

fn remove_null_types(value: &mut Value) {
    match value {
        Value::Mapping(map) => {
            // Check if this is a type array with null
            if let Some(Value::Sequence(types)) = map.get(Value::String("oneOf".to_string())) {
                let has_null = types
                    .iter()
                    .filter_map(|t| t.as_mapping())
                    .filter_map(|t| t.get(Value::String("type".to_string())))
                    .filter_map(|t| t.as_str())
                    .any(|v| v.eq("null"));
                let other_types: Vec<_> = types
                    .iter()
                    .filter(|v| {
                        v.as_mapping()
                            .and_then(|t| t.get(Value::String("type".to_string())))
                            .and_then(|t| t.as_str())
                            != Some("null")
                    })
                    .cloned()
                    .collect();

                if has_null && !other_types.is_empty() {
                    // Replace type array with single type and add nullable
                    if other_types.len() == 1 {
                        map.remove(Value::String("oneOf".to_string()));
                        let value = other_types.first().unwrap().as_mapping().unwrap();
                        map.insert(
                            Value::String("$ref".to_string()),
                            value
                                .get(Value::String("$ref".to_string()))
                                .unwrap()
                                .clone(),
                        );
                    }
                }
            }

            // Recursively process all values
            for (_, v) in map.iter_mut() {
                remove_null_types(v);
            }
        }
        Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                remove_null_types(v);
            }
        }
        _ => {}
    }
}

fn replace_wildcard_content_types(value: &mut Value) {
    match value {
        Value::Mapping(map) => {
            // Check if this mapping has a key "*/*" and replace it
            if map.contains_key(Value::String("*/*".to_string())) {
                if let Some(content) = map.remove(Value::String("*/*".to_string())) {
                    map.insert(
                        Value::String("application/octet-stream".to_string()),
                        content,
                    );
                }
            }

            // Recursively process all values
            for (_, v) in map.iter_mut() {
                replace_wildcard_content_types(v);
            }
        }
        Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                replace_wildcard_content_types(v);
            }
        }
        _ => {}
    }
}

fn convert_array_query_params_to_string(value: &mut Value) {
    match value {
        Value::Mapping(map) => {
            // Check if this is a query parameter with array type
            if let Some(Value::String(location)) = map.get(Value::String("in".to_string())) {
                if location == "query" {
                    if let Some(Value::Mapping(schema)) =
                        map.get_mut(Value::String("schema".to_string()))
                    {
                        if let Some(Value::String(type_str)) =
                            schema.get(Value::String("type".to_string()))
                        {
                            if type_str == "array" {
                                // Convert array to comma-separated string
                                // Remove the items field and change type to string
                                schema.remove(Value::String("items".to_string()));
                                schema.insert(
                                    Value::String("type".to_string()),
                                    Value::String("string".to_string()),
                                );
                                schema.insert(
                                    Value::String("description".to_string()),
                                    Value::String("Comma-separated values".to_string()),
                                );
                            }
                        }
                    }
                }
            }

            // Recursively process all values
            for (_, v) in map.iter_mut() {
                convert_array_query_params_to_string(v);
            }
        }
        Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                convert_array_query_params_to_string(v);
            }
        }
        _ => {}
    }
}
