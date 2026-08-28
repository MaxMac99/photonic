use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use snafu::{ResultExt, Whatever};

use crate::{auth::JwtUserClaims, settings::AuthSettings};

pub async fn setup_auth(settings: &AuthSettings) -> Result<Authorizer<JwtUserClaims>, Whatever> {
    let validation = Validation::new().aud(std::slice::from_ref(&settings.client_id));

    // Check if we're in test mode (using JWT_SECRET env var)
    if let Some(secret) = settings.jwt_secret.as_ref() {
        // Test mode: use symmetric secret (HS256)
        tracing::warn!(
            "Using JWT_SECRET for authentication (TEST MODE ONLY - not for production!)"
        );
        JwtAuthorizer::from_secret(secret)
            .validation(validation)
            .check(|claims: &JwtUserClaims| claims.get_username().is_some())
            .build()
            .await
            .whatever_context("Could not create JWT Authorizer from secret")
    } else {
        // Production mode: use JWKS URL
        tracing::info!("Using JWKS URL for authentication: {}", settings.jwks_url);
        JwtAuthorizer::from_jwks_url(settings.jwks_url.as_str())
            .validation(validation)
            .check(|claims: &JwtUserClaims| claims.get_username().is_some())
            .build()
            .await
            .whatever_context("Could not create JWT Authorizer from JWKS")
    }
}
