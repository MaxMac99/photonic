use std::error::Error;

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
// - Configuration values exposure
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
    assert!(
        !response.client_id.is_empty(),
        "Should have client_id field"
    );
    assert!(
        !response.token_url.is_empty(),
        "Should have token_url field"
    );
    assert!(
        !response.authorize_url.is_empty(),
        "Should have authorize_url field"
    );

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
