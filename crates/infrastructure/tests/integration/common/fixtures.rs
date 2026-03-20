use std::path::PathBuf;

use byte_unit::Byte;
use domain::user::{QuotaState, User};
use fake::{Fake, Faker};
use rstest::*;
use uuid::Uuid;

use crate::integration::test_app::TestApp;

/// Reusable fixture for getting the shared test app instance
/// The app is created once on first use and reused across all tests
#[fixture]
pub async fn app() -> TestApp {
    TestApp::new().await
}

#[fixture]
pub fn user(#[default(10_000_000_000)] quota_limit: u64) -> User {
    User {
        id: Uuid::new_v4(),
        version: 1,
        username: Faker.fake(),
        email: Faker.fake(),
        quota: QuotaState::new_unchecked(Byte::from_u64(0), Byte::from_u64(quota_limit)),
    }
}

#[derive(Debug, Clone)]
pub struct ImageFixture {
    pub filename: &'static str,
    pub data: Vec<u8>,
}

#[fixture]
pub fn image(#[default("IMG_4598.HEIC")] filename: &'static str) -> ImageFixture {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("images");
    path.push(filename);

    let data = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to load test image fixture '{}': {}",
            path.display(),
            e
        )
    });
    ImageFixture { filename, data }
}
