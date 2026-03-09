use std::{env, path::Path};

use xtask::openapi::{convert_openapi, generate_openapi_client, generate_openapi_spec};

#[tokio::main]
async fn main() {
    eprintln!("[build.rs] Starting photonic-client build script");

    println!("cargo::rerun-if-changed=../photonic/src/infrastructure/api");
    println!("cargo::rerun-if-changed=../xtask/src");

    let out_dir = env::var("OUT_DIR").unwrap();
    eprintln!("[build.rs] OUT_DIR: {}", out_dir);

    let converted_spec = Path::new(&out_dir)
        .join("openapi-3.0.yaml")
        .to_str()
        .unwrap()
        .to_string();
    let openapi_path = "../../openapi.yaml";

    eprintln!("[build.rs] Generating OpenAPI spec at: {}", openapi_path);
    generate_openapi_spec(openapi_path)
        .await
        .expect("Failed to generate OpenAPI spec");
    eprintln!("[build.rs] OpenAPI spec generated successfully");

    eprintln!("[build.rs] Converting OpenAPI spec to: {}", converted_spec);
    convert_openapi(openapi_path, &converted_spec).expect("Failed to convert OpenAPI");
    eprintln!("[build.rs] OpenAPI spec converted successfully");

    let client_path = Path::new(&out_dir).join("photonic_client.rs");
    eprintln!("[build.rs] Generating client at: {:?}", client_path);
    generate_openapi_client(&converted_spec, &client_path);
    eprintln!("[build.rs] Client generated successfully");

    eprintln!("[build.rs] Build script completed");
}
