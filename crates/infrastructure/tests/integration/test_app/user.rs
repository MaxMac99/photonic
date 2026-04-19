use std::env;

use chrono::{Duration, Utc};
use domain::user::User;
use infrastructure::auth::JwtUserClaims;
use jsonwebtoken::{encode, Header};
use serde::{Deserialize, Serialize};

use crate::integration::test_app::TestApp;

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    #[serde(flatten)]
    user_claims: JwtUserClaims,
    // Standard JWT claims
    aud: String,
    exp: usize,
    iat: usize,
}

impl TestApp {
    /// Generate a JWT token for a test user
    pub fn create_jwt_token(&self, user: &User) -> String {
        let user_claims = JwtUserClaims {
            sub: user.id,
            email: user.email.clone(),
            name: Some(user.username.clone()),
            given_name: None,
            preferred_username: None,
            nickname: None,
            quota: Some(user.quota.limit()),
        };

        let claims = TokenClaims {
            user_claims,
            aud: env::var("OAUTH_CLIENT_ID").unwrap_or_else(|_| "test-client-id".to_string()),
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key).expect("Failed to create JWT token")
    }
}
