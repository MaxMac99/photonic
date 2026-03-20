pub mod medium;
pub mod user;

use std::env;

use domain::user::User;
use dotenv::dotenv;
use infrastructure::config::GlobalConfig;
use infrastructure::{run_server, setup_test_tracing, ServerHandle};
use jsonwebtoken::EncodingKey;
use photonic_client::Client as GeneratedClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Test application using the real application server
pub struct TestApp {
    server: Option<ServerHandle>,
    pub base_url: String,
    pub encoding_key: EncodingKey,
    pub db_pool: PgPool,
}

impl TestApp {
    /// Start a new test application with real server on random port
    pub async fn new() -> Self {
        // Load .env file for test configuration
        dotenv().ok();

        // Set up test logging
        setup_test_tracing();

        // Load configuration
        let config = GlobalConfig::load()
            .await
            .expect("Failed to load configuration");

        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret-key".to_string());
        let key = EncodingKey::from_secret(secret.as_bytes());

        // Connect to database for test utilities
        let db_pool = PgPool::connect(&config.database.url)
            .await
            .expect("Failed to connect to database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        // Start server on random port (0 = OS assigns)
        let server = run_server(config, Some(0))
            .await
            .expect("Failed to start server");

        let base_url = format!("http://{}", server.addr);

        Self {
            server: Some(server),
            base_url,
            encoding_key: key,
            db_pool,
        }
    }

    pub fn client(&self) -> GeneratedClient {
        let raw_client = Client::new();
        GeneratedClient::new_with_client(&self.base_url, raw_client)
    }

    pub fn client_with_user(&self, user: &User) -> GeneratedClient {
        let token = self.create_jwt_token(user);
        let raw_client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", token)
                        .parse()
                        .expect("Failed to parse authorization header"),
                );
                headers
            })
            .build()
            .expect("Failed to build client with user");

        GeneratedClient::new_with_client(&self.base_url, raw_client)
    }

    /// Clean up test data from the database
    pub async fn cleanup(&self) {
        sqlx::query("TRUNCATE users, albums, media, medium_items, locations, media_tags CASCADE")
            .execute(&self.db_pool)
            .await
            .expect("Failed to clean test database");
    }

    /// Shut down the server and release all resources.
    /// Call this at the end of each test (before cleanup or after).
    pub async fn shutdown(&mut self) {
        if let Some(server) = self.server.take() {
            server.shutdown().await;
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // If shutdown() wasn't called explicitly, force-shutdown via the channel.
        // take() ensures the ServerHandle is dropped, closing the Sender.
        if self.server.is_some() {
            tracing::warn!("TestApp dropped without explicit shutdown, forcing shutdown");
            self.server.take();
        }
    }
}
