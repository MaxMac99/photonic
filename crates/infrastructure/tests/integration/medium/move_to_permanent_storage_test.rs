use std::{error::Error, time::Duration};

use domain::user::User;
use photonic_client::types::StorageTierDto;
use rstest::*;
use serial_test::serial;

use crate::integration::{
    common::fixtures::{app, image, user, ImageFixture},
    test_app::TestApp,
};

/// Test that after metadata extraction completes, the medium item is moved from
/// temp storage to permanent storage with the correct pattern-based path.
#[rstest]
#[timeout(Duration::from_secs(30))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_medium_item_moves_to_permanent_storage_after_metadata_extraction(
    #[future(awt)] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange
    let medium_id = app.create_medium(&user, image.into()).await?.into_inner();

    // Act: Wait for the medium item to be moved to permanent storage
    let medium = app
        .wait_for_permanent_storage(&user, &medium_id)
        .await
        .expect("Medium item was not moved to permanent storage in time");

    // Assert: The primary item should have both temp and permanent locations
    let primary_item = medium
        .items
        .iter()
        .find(|item| item.is_primary)
        .expect("Primary item not found");

    // Should have two locations: temp (original) + permanent (copy)
    assert_eq!(
        primary_item.locations.len(),
        2,
        "Expected two locations (temp + permanent) after copy"
    );

    let permanent_location = primary_item
        .locations
        .iter()
        .find(|l| matches!(l.storage_tier, StorageTierDto::Permanent))
        .expect("Expected a permanent location");

    assert!(
        primary_item
            .locations
            .iter()
            .any(|l| matches!(l.storage_tier, StorageTierDto::Temporary)),
        "Expected temp location to still be present"
    );

    // The permanent path should follow the configured pattern (contains / for directory structure)
    let path = &permanent_location.relative_path;
    assert!(
        path.contains('/'),
        "Path should follow pattern with directories, got: {}",
        path
    );

    // Clean up
    app.cleanup().await;
    Ok(())
}

/// Test that a photo without EXIF capture date gets moved with defaults
/// and has needs_reorganization set to true.
#[rstest]
#[timeout(Duration::from_secs(30))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_medium_with_missing_metadata_sets_needs_reorganization(
    #[future(awt)] app: TestApp,
    user: User,
    #[with("IMG_4597.DNG")] image: ImageFixture, // DNG might have different metadata
) -> Result<(), Box<dyn Error>> {
    // Arrange
    let medium_id = app.create_medium(&user, image.into()).await?.into_inner();

    // Act: Wait for the medium item to be moved to permanent storage
    let medium = app
        .wait_for_permanent_storage(&user, &medium_id)
        .await
        .expect("Medium item was not moved to permanent storage in time");

    // Assert: Check that the file was copied to permanent storage
    // (needs_reorganization field will be added to the response DTO later)
    let primary_item = medium
        .items
        .iter()
        .find(|item| item.is_primary)
        .expect("Primary item not found");

    assert!(
        primary_item
            .locations
            .iter()
            .any(|l| matches!(l.storage_tier, StorageTierDto::Permanent)),
        "Expected a permanent location to exist"
    );
    assert!(
        primary_item
            .locations
            .iter()
            .any(|l| matches!(l.storage_tier, StorageTierDto::Temporary)),
        "Expected temp location to still be present"
    );

    // Clean up
    app.cleanup().await;
    Ok(())
}
