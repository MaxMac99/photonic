use std::error::Error;

use domain::user::User;
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;

use crate::integration::{
    common::fixtures::{app, image, user, ImageFixture},
    test_app::TestApp,
};

// ============================================================================
// CREATE MEDIUM TESTS - POST /api/v1/medium
// ============================================================================
// This file tests the medium creation endpoint, focusing on:
// - Successful creation with valid data
// - Different file types and formats
// - Authorization failures
// - Business logic failures (quota, validation)
// ============================================================================

#[rstest]
#[case::heic_format("IMG_4598.HEIC")]
#[case::dng_format("IMG_4597.DNG")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_create_medium_success(
    #[future] app: TestApp,
    user: User,
    #[case] fixture_name: &str,
    #[with(fixture_name)] image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Act: Create a medium using the helper
    let response = app.create_medium(&user, image.into()).await?;

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Expected 201 CREATED status"
    );
    assert!(!response.is_nil(), "Medium ID should not be nil");

    app.cleanup().await;
    Ok(())
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_create_medium_with_custom_filename(
    #[future] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange
    let fixture_name = "custom_name.heic";
    let image = ImageFixture {
        filename: fixture_name,
        data: image.data,
    };

    // Act: Create a medium using the helper
    let response = app.create_medium(&user, image.into()).await?;

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Expected 201 CREATED status"
    );
    assert!(!response.is_nil(), "Medium ID should not be nil");

    app.cleanup().await;
    Ok(())
}

// === Authorization Tests ===

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_create_medium_without_auth_fails(
    #[future(awt)] app: TestApp,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Act: Try to create medium without auth token
    let response = app
        .client()
        .create_medium(
            None,
            None,
            None,
            None,
            image.filename,
            None,
            None,
            None,
            image.data,
        )
        .await;

    // Assert: Should be unauthorized
    if let Err(photonic_client::Error::UnexpectedResponse(response)) = response {
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 UNAUTHORIZED without auth token"
        );
    } else {
        return Err("Expected UnexpectedResponse error".into());
    }

    app.cleanup().await;
    Ok(())
}

// === Business Logic Failure Tests ===

#[rstest]
#[case::no_quota(0, "no quota available")]
#[case::insufficient_quota(1, "file exceeds available quota")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_create_medium_with_quota_fails(
    #[case] available_quota: u64,
    #[case] reason: &str,
    #[future(awt)] app: TestApp,
    #[with(available_quota)] user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Act: Create a medium using the helper
    let response = app.create_medium(&user, image.into()).await;

    // Assert: Should be forbidden due to quota
    if let Err(photonic_client::Error::UnexpectedResponse(response)) = response {
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Expected 403 FORBIDDEN when {reason}"
        );
    } else {
        return Err("Expected UnexpectedResponse error".into());
    }

    app.cleanup().await;
    Ok(())
}
