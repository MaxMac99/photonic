use std::{env, error::Error};

use composition::{run_server, setup_test_tracing, GlobalConfig};
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;

use crate::integration::{common::fixtures::app, test_app::TestApp};
// ============================================================================
// SYSTEM INFO TESTS - GET /api/v1/system
// ============================================================================
// This file tests the system information endpoint, focusing on:
// - Public accessibility (no auth required)
// - Response structure and required fields
// - Configuration values exposure (nulls when OIDC is disabled)
// ============================================================================

#[rstest]
#[tokio::test]
#[serial]
#[awt]
async fn test_system_info_success(#[future] app: TestApp) -> Result<(), Box<dyn Error>> {
    // Act: Get system info without authentication
    let response = app.client().system_info().await?;

    // Verify all required fields are present
    assert_eq!(response.status(), StatusCode::OK, "Should return 200 OK");
    assert!(!response.version.is_empty(), "Should have version field");
    let client_id = response
        .client_id
        .as_ref()
        .expect("client_id should be set when OIDC is configured");
    let token_url = response
        .token_url
        .as_ref()
        .expect("token_url should be set when OIDC is configured");
    let authorize_url = response
        .authorize_url
        .as_ref()
        .expect("authorize_url should be set when OIDC is configured");
    assert!(!client_id.is_empty(), "Should have client_id field");
    assert!(!token_url.is_empty(), "Should have token_url field");
    assert!(!authorize_url.is_empty(), "Should have authorize_url field");

    app.cleanup().await;
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[awt]
async fn test_system_info_version_format(#[future] app: TestApp) -> Result<(), Box<dyn Error>> {
    // Act: Get system info
    let response = app.client().system_info().await?;

    // Assert
    assert_eq!(response.status(), StatusCode::OK, "Should return 200 OK");
    assert!(!response.version.is_empty(), "Version should not be empty");

    // Version should follow semantic versioning pattern (e.g., "0.1.0")
    let version_parts: Vec<&str> = response.version.split('.').collect();
    assert!(
        version_parts.len() >= 2,
        "Version should have at least major.minor format"
    );

    app.cleanup().await;
    Ok(())
}

/// Starts a server with all authentication configuration removed and verifies
/// it boots without OIDC, reports no auth settings in the info endpoint, and
/// still rejects anonymous access to protected endpoints.
#[tokio::test]
#[serial]
async fn test_system_info_oidc_disabled() -> Result<(), Box<dyn Error>> {
    setup_test_tracing();

    // Preserve the environment so subsequent serial tests keep working
    let auth_vars = [
        "OAUTH_CLIENT_ID",
        "OAUTH_JWKS_URL",
        "OAUTH_TOKEN_URL",
        "OAUTH_AUTHORIZE_URL",
        "JWT_SECRET",
    ];
    let saved: Vec<(String, Option<String>)> = auth_vars
        .iter()
        .map(|var| (var.to_string(), env::var(var).ok()))
        .collect();
    for var in auth_vars {
        env::remove_var(var);
    }

    let result = async {
        // Nothing is configured: OIDC must be disabled and the server must start
        let config = GlobalConfig::load()
            .await
            .expect("config should load without any auth settings");
        let server = run_server(config, Some(0))
            .await
            .expect("server should start without OIDC");
        let base_url = format!("http://{}", server.addr);
        let client = reqwest::Client::new();

        // Info endpoint reports no auth configuration
        let info: serde_json::Value = client
            .get(format!("{base_url}/api/v1/system"))
            .send()
            .await?
            .json()
            .await?;
        assert!(
            info["version"].as_str().map(|v| !v.is_empty()) == Some(true),
            "version should still be present: {info}"
        );
        assert!(
            info["client_id"].is_null(),
            "client_id should be null: {info}"
        );
        assert!(
            info["token_url"].is_null(),
            "token_url should be null: {info}"
        );
        assert!(
            info["authorize_url"].is_null(),
            "authorize_url should be null: {info}"
        );

        // Protected endpoints still reject requests without claims
        let status = client
            .get(format!("{base_url}/api/v1/medium"))
            .send()
            .await?
            .status();
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "medium endpoints require claims even with OIDC disabled"
        );

        server.shutdown().await;
        Ok(())
    }
    .await;

    for (var, value) in saved {
        if let Some(value) = value {
            env::set_var(&var, value);
        }
    }

    result
}
