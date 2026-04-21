use std::{fs, path::Path};

pub fn generate_openapi_client(openapi_path: &str, output_path: &Path) {
    eprintln!("Generating typed client from OpenAPI 3.0.3 spec...");

    let spec_content =
        fs::read_to_string(&openapi_path).expect("Failed to read converted OpenAPI spec");

    let spec: openapiv3::OpenAPI =
        serde_yaml::from_str(&spec_content).expect("Failed to parse converted OpenAPI YAML");

    let mut generator = progenitor::Generator::default();
    let tokens = generator
        .generate_tokens(&spec)
        .expect("Failed to generate client code from OpenAPI spec");

    let ast = syn::parse2(tokens).expect("Failed to parse generated code");
    let code = prettyplease::unparse(&ast);

    fs::write(&output_path, code).expect("Failed to write generated client");

    println!(
        "cargo:rustc-env=PHOTONIC_CLIENT_PATH={}",
        output_path.display()
    );

    eprintln!("✓ Generated client at {}", output_path.display());
}
