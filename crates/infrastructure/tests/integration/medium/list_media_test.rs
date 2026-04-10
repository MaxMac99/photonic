use std::{error::Error, time::Duration};

use domain::user::User;
use reqwest::StatusCode;
use rstest::*;
use serial_test::serial;
use uuid::Uuid;

use crate::integration::{
    common::{
        fixtures::{app, image, user, ImageFixture},
        polling::{poll_until, PollingConfig},
    },
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
    let expected_id = image_id.into_inner();

    // Act + Assert: Poll until the projection has updated the read model
    let response = poll_until(
        || {
            let client = app.client_with_user(&user);
            async move {
                let response = client
                    .get_all_media(None, None, None, None, None, None, None, None, None)
                    .await
                    .ok()?;
                if response.len() == 1 {
                    Some(response)
                } else {
                    None
                }
            }
        },
        PollingConfig::quick("medium to appear in list"),
    )
    .await
    .expect("Medium should appear in list");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.first().map(|m| m.id),
        Some(expected_id),
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

    // Act + Assert: Poll until all 5 media appear
    let response = poll_until(
        || {
            let client = app.client_with_user(&user);
            async move {
                let response = client
                    .get_all_media(None, None, None, None, None, None, None, None, None)
                    .await
                    .ok()?;
                if response.len() == 5 {
                    Some(response)
                } else {
                    None
                }
            }
        },
        PollingConfig::quick("all 5 media to appear in list"),
    )
    .await
    .expect("All 5 media should appear in list");

    assert_eq!(response.status(), StatusCode::OK);

    let response_ids: Vec<Uuid> = response.iter().map(|m| m.id).collect();
    for media_id in &media_ids {
        assert!(
            response_ids.contains(media_id),
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
    let _user2_medium = app
        .create_medium(&user2, image.clone().into())
        .await?
        .into_inner();

    // Act + Assert: Poll until User 1 sees their medium
    let response = poll_until(
        || {
            let client = app.client_with_user(&user1);
            async move {
                let response = client
                    .get_all_media(None, None, None, None, None, None, None, None, None)
                    .await
                    .ok()?;
                if response.len() == 1 {
                    Some(response)
                } else {
                    None
                }
            }
        },
        PollingConfig::quick("user1 medium to appear in list"),
    )
    .await
    .expect("User 1 should see their medium");

    assert_eq!(response.status(), StatusCode::OK);

    let response_ids: Vec<Uuid> = response.iter().map(|m| m.id).collect();
    assert!(
        response_ids.contains(&user1_medium),
        "User 1 media should be in response"
    );

    app.cleanup().await;
    Ok(())
}
