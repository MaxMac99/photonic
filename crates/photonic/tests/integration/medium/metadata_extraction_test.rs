use std::{error::Error, time::Duration};

use photonic::domain::user::User;
use rstest::*;
use serial_test::serial;

use crate::integration::{
    common::fixtures::{app, image, user, ImageFixture},
    test_app::TestApp,
};

#[rstest]
#[timeout(Duration::from_secs(15))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_metadata_extraction_enriches_medium(
    #[future(awt)] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange
    let medium_id = app.create_medium(&user, image.into()).await?.into_inner();

    // Act: Wait for medium to be enriched with metadata
    let medium = app
        .wait_for_medium_enrichment(&user, &medium_id)
        .await
        .expect("Medium enrichment timed out");

    // Assert: Verify medium was enriched with metadata
    assert!(medium.camera_make.is_some());

    // Clean up
    app.cleanup().await;
    Ok(())
}

#[rstest]
#[timeout(Duration::from_secs(15))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_metadata_endpoint_returns_full_metadata(
    #[future(awt)] app: TestApp,
    user: User,
    image: ImageFixture,
) -> Result<(), Box<dyn Error>> {
    // Arrange
    let medium_id = app.create_medium(&user, image.into()).await?.into_inner();

    // Act: Wait for metadata to be available via the dedicated endpoint
    let metadata = app
        .wait_for_metadata(&user, &medium_id)
        .await
        .expect("Metadata endpoint timed out");

    // Assert: Verify full metadata is returned
    assert!(metadata.camera_info.is_some());
    let camera_info = metadata.camera_info.unwrap();
    assert!(camera_info.make.is_some());

    // Clean up
    app.cleanup().await;
    Ok(())
}
