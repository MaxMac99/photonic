use std::error::Error;

use photonic::domain::user::User;
use reqwest::StatusCode;
use rstest::*;
use serial_test::serial;
use uuid::Uuid;

use crate::integration::{
    common::fixtures::{app, image, user, ImageFixture},
    test_app::TestApp,
};

// ============================================================================
// LIST MEDIA TESTS - GET /api/v1/medium
// ============================================================================
// This file tests the media listing endpoint, focusing on:
// - Listing empty collections
// - Listing single items
// - Listing multiple items
// - Different file types in listings
// - Pagination and filtering (when implemented)
// ============================================================================

// === Empty List Tests ===

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_list_media_when_empty(
    #[future] app: TestApp,
    user: User,
) -> Result<(), Box<dyn Error>> {
    // Act: List media when none exist
    let response = app
        .client_with_user(&user)
        .get_all_media(None, None, None, None, None, None, None, None, None)
        .await?;

    // Assert: Should return empty list
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.len(), 0, "Should have no media");

    app.cleanup().await;
    Ok(())
}

// === Single Item Tests ===

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_list_media_with_single_item(
    #[future] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange: Create a single medium
    let image_id = app.create_medium(&user, image.into()).await?;

    // Act: List all media
    let response = app
        .client_with_user(&user)
        .get_all_media(None, None, None, None, None, None, None, None, None)
        .await?;

    // Assert: Should return the single medium
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.len(), 1, "Should have one medium");
    assert_eq!(
        response.first().map(|m| m.id),
        Some(image_id.into_inner()),
        "Medium ID should match"
    );

    app.cleanup().await;
    Ok(())
}

// === Multiple Items Tests ===

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_list_media_with_multiple_items(
    #[future] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange
    let mut media_ids = Vec::new();
    for _ in 0..5 {
        let media_id = app.create_medium(&user, image.clone().into()).await?;
        media_ids.push(media_id.into_inner());
    }

    // Act: List all media
    let response = app
        .client_with_user(&user)
        .get_all_media(None, None, None, None, None, None, None, None, None)
        .await?;

    // Assert: Should return all 5 media
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.len(), 5, "Should have 5 media");

    // Verify all IDs are present
    let response_ids: Vec<Uuid> = response.iter().map(|m| m.id).collect();

    for media_id in media_ids {
        assert!(
            response_ids.contains(&media_id),
            "Media ID {} should be in response",
            media_id
        );
    }

    app.cleanup().await;
    Ok(())
}

// === Different File Types Tests ===

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_list_media_with_different_file_types(
    #[future] app: TestApp,
    user: User,
    #[from(image)]
    #[with("IMG_4598.HEIC")]
    heic: ImageFixture,
    #[from(image)]
    #[with("IMG_4597.DNG")]
    dng: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange: Create media with different file types
    let mut media_ids = Vec::new();
    for fixture in [heic, dng] {
        let media_id = app.create_medium(&user, fixture.into()).await?;
        media_ids.push(media_id.into_inner());
    }

    // Act: List all media
    let response = app
        .client_with_user(&user)
        .get_all_media(None, None, None, None, None, None, None, None, None)
        .await?;

    // Assert: Should return both different file types
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.len(), 2, "Should have 2 media");

    app.cleanup().await;
    Ok(())
}

// === User Isolation Tests ===

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_list_media_only_shows_own_media(
    #[future(awt)] app: TestApp,
    #[from(user)] user1: User,
    #[from(user)] user2: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange: Create two users with their own media
    let user1_medium = app
        .create_medium(&user1, image.clone().into())
        .await?
        .into_inner();
    let user2_medium = app
        .create_medium(&user2, image.clone().into())
        .await?
        .into_inner();

    // Act: User 1 lists their media
    let response = app
        .client_with_user(&user1)
        .get_all_media(None, None, None, None, None, None, None, None, None)
        .await?;

    // Assert: User 1 should only see their 3 media
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.len(), 1, "User 1 should have 1 medium");

    let response_ids: Vec<Uuid> = response.iter().map(|m| m.id).collect();

    // Verify User 1's media are present
    assert!(
        response_ids.contains(&user1_medium),
        "User 1 media should be in response"
    );

    // Verify User 2's media are NOT present
    assert!(
        !response_ids.contains(&user2_medium),
        "User 2 media should NOT be in User 1's response"
    );

    app.cleanup().await;
    Ok(())
}
