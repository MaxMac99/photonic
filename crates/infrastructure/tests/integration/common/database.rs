use std::env;

use sqlx::PgPool;

/// Get or create test database pool
/// Assumes Nix test shell is active with TEST_DATABASE_URL set
pub async fn get_test_pool() -> PgPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set (run in `nix develop .#test`)");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Ensure migrations are up to date
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Clean all data from test database
pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query(
        "TRUNCATE users, albums, media, medium_items, locations, media_tags, \
         tasks, metadata, event_streams, events, snapshots, projection_checkpoints CASCADE",
    )
    .execute(pool)
    .await
    .expect("Failed to clean test database");
}

/// Helper to run a test with a fresh database
pub async fn with_test_db<F, Fut>(test: F)
where
    F: FnOnce(PgPool) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let pool = get_test_pool().await;
    cleanup_test_db(&pool).await;
    test(pool).await;
}
