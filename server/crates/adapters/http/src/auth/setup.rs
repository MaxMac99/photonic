use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use snafu::{OptionExt, ResultExt, Whatever};

use crate::{auth::JwtUserClaims, settings::AuthSettings};

pub async fn setup_auth(settings: &AuthSettings) -> Result<Authorizer<JwtUserClaims>, Whatever> {
    let validation = match settings.client_id.as_ref() {
        Some(client_id) => Validation::new().aud(std::slice::from_ref(client_id)),
        None => Validation::new(),
    };

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
        let jwks_url = settings
            .jwks_url
            .as_deref()
            .whatever_context("jwks_url is required when JWT_SECRET is not set")?;
        tracing::info!("Using JWKS URL for authentication: {}", jwks_url);
        JwtAuthorizer::from_jwks_url(jwks_url)
            .validation(validation)
            .check(|claims: &JwtUserClaims| claims.get_username().is_some())
            .build()
            .await
            .whatever_context("Could not create JWT Authorizer from JWKS")
    }
}
