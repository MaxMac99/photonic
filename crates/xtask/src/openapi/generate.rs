use std::fs;

use infrastructure::api::router::create_api;
use snafu::{ResultExt, Whatever};

pub async fn generate_openapi_spec(output: &str) -> Result<(), Whatever> {
    eprintln!("Generating OpenAPI spec to {}", output);

    let openapi = create_api();
    let yaml = openapi.to_yaml().whatever_context("OpenAPI spec yaml")?;

    fs::write(output, yaml).whatever_context("Failed to write YAML file")?;

    eprintln!("✓ OpenAPI spec written to {}", output);
    Ok(())
}
