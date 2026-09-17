use std::error::Error;

use ::user::domain::User;
use photonic_client::types::MediumItemTypeDto;
use reqwest::StatusCode;
use rstest::rstest;
use serial_test::serial;

use crate::integration::{
    common::fixtures::{app, image, user, ImageFixture},
    test_app::{medium::AddMediumItemRequest, TestApp},
};

// ============================================================================
// THUMBNAIL PIPELINE TESTS - issue #1
// ============================================================================
// Client uploads original + thumbnail variants (tiny/small) via sequential
// requests. Server stores previews as MediumItems in the Cache tier,
// computes the thumbhash from the tiny variant, and serves both raw items
// and variant-matched previews.
// ============================================================================

fn thumbnail_request(
    filename: &str,
    variant: &str,
    width: Option<u32>,
    height: Option<u32>,
    data: Vec<u8>,
) -> AddMediumItemRequest {
    AddMediumItemRequest {
        format: "preview".to_string(),
        variant: Some(variant.to_string()),
        width,
        height,
        filename: filename.to_string(),
        content_type: "image/jpeg".to_string(),
        body: data,
    }
}

fn load_fixture(name: &str) -> Vec<u8> {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/images");
    path.push(name);
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture '{}': {}", path.display(), e))
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_upload_thumbnails_stored_and_retrievable(
    #[future] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    let tiny_data = load_fixture("thumb_tiny.jpg");
    let small_data = load_fixture("thumb_small.jpg");

    // Arrange: upload the original
    let medium_id = app.create_medium(&user, image.into()).await?.into_inner();

    // Act: upload thumbnail variants
    let tiny_response = app
        .add_medium_item(
            &user,
            &medium_id,
            thumbnail_request("tiny.jpg", "tiny", Some(150), Some(112), tiny_data.clone()),
        )
        .await;
    let small_response = app
        .add_medium_item(
            &user,
            &medium_id,
            thumbnail_request(
                "small.jpg",
                "small",
                Some(600),
                Some(450),
                small_data.clone(),
            ),
        )
        .await;

    // Assert: both uploads accepted
    let tiny_item_id = tiny_response
        .expect("tiny upload should return 201")
        .into_inner();
    let small_item_id = small_response
        .expect("small upload should return 201")
        .into_inner();
    assert_ne!(tiny_item_id, small_item_id);

    // Assert: medium detail contains three items and the thumbhash
    let medium = app
        .wait_for_thumbhash(&user, &medium_id)
        .await
        .expect("medium should have a thumbhash after the tiny upload");
    assert_eq!(medium.items.len(), 3, "original + two thumbnails");
    assert!(
        medium.thumbhash.is_some(),
        "detail response should include the thumbhash"
    );

    // Assert: previews are stored in the Cache tier
    let preview_items: Vec<_> = medium
        .items
        .iter()
        .filter(|i| i.medium_item_type == MediumItemTypeDto::Preview)
        .collect();
    assert_eq!(preview_items.len(), 2);
    for item in preview_items {
        assert!(
            item.locations.iter().any(|l| matches!(
                l.storage_tier,
                photonic_client::types::StorageTierDto::Cache
            )),
            "preview items must be stored in the Cache tier"
        );
    }

    // Assert: /preview without size hint serves the small variant
    let response = app
        .get_raw(&user, &format!("/api/v1/medium/{medium_id}/preview"), &[])
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/jpeg");
    let body = response.bytes().await?;
    assert_eq!(
        body.len(),
        small_data.len(),
        "default preview should be the small variant"
    );

    // Assert: /preview?width=200 serves the tiny variant
    let response = app
        .get_raw(
            &user,
            &format!("/api/v1/medium/{medium_id}/preview"),
            &[("width", "200".to_string())],
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.bytes().await?;
    assert_eq!(
        body.len(),
        tiny_data.len(),
        "requested 200px edge should serve the tiny variant"
    );

    // Assert: raw item endpoint returns the uploaded bytes
    let response = app
        .get_raw(
            &user,
            &format!("/api/v1/medium/{medium_id}/item/{tiny_item_id}/raw"),
            &[],
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.bytes().await?;
    assert_eq!(
        body.as_ref(),
        tiny_data.as_slice(),
        "raw item should return the uploaded thumbnail"
    );

    // Assert: list endpoint carries the denormalized thumbhash
    let list = app
        .client_with_user(&user)
        .get_all_media(None, None, None, None, None, None, None, None, None)
        .await?
        .into_inner();
    let listed = list
        .into_iter()
        .find(|m| m.id == medium_id)
        .expect("medium should appear in list");
    assert!(
        listed.thumbhash.is_some(),
        "list response should include the thumbhash"
    );

    app.cleanup().await;
    Ok(())
}

#[rstest]
#[case::oversized_dimensions(Some(1500), Some(1500), "dimensions exceed the variant limit")]
#[case::missing_dimensions(None, None, "dimensions are required")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_upload_invalid_thumbnail_fails(
    #[future] app: TestApp,
    user: User,
    image: ImageFixture,
    #[case] width: Option<u32>,
    #[case] height: Option<u32>,
    #[case] reason: &str,
) -> Result<(), Box<dyn Error>> {
    let tiny_data = load_fixture("thumb_tiny.jpg");

    // Arrange
    let medium_id = app.create_medium(&user, image.into()).await?.into_inner();

    // Act
    let response = app
        .add_medium_item(
            &user,
            &medium_id,
            thumbnail_request("tiny.jpg", "tiny", width, height, tiny_data.clone()),
        )
        .await;

    // Assert
    match response {
        Err(photonic_client::Error::UnexpectedResponse(response)) => {
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "Expected 400 for {reason}"
            );
        }
        other => {
            return Err(format!("Expected 400 BAD_REQUEST for {reason}, got {other:?}").into())
        }
    }

    app.cleanup().await;
    Ok(())
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
#[awt]
async fn test_upload_preview_to_missing_medium_fails(
    #[future] app: TestApp,
    user: User,
) -> Result<(), Box<dyn Error>> {
    let tiny_data = load_fixture("thumb_tiny.jpg");

    // Arrange
    let missing_id = uuid::Uuid::new_v4();

    // Act
    let response = app
        .add_medium_item(
            &user,
            &missing_id,
            thumbnail_request("tiny.jpg", "tiny", Some(150), Some(112), tiny_data),
        )
        .await;

    // Assert
    match response {
        Err(photonic_client::Error::UnexpectedResponse(response)) => {
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }
        other => return Err(format!("Expected 404 NOT_FOUND, got {other:?}").into()),
    }

    app.cleanup().await;
    Ok(())
}
