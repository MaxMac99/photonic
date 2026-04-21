use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use snafu::{ResultExt, Whatever};
use tokio::{signal, sync::mpsc::Receiver};
use tracing::log::debug;

use crate::{auth::JwtUserClaims, config::ServerConfig};

pub async fn shutdown_signal_with_external_signal(mut died_receiver: Receiver<bool>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
        _ = died_receiver.recv() => {},
    }

    debug!("signal received, starting graceful shutdown");
}

pub async fn setup_auth(
    server_config: &ServerConfig,
) -> Result<Authorizer<JwtUserClaims>, Whatever> {
    let validation = Validation::new().aud(std::slice::from_ref(&server_config.client_id));

    // Check if we're in test mode (using JWT_SECRET env var)
    if let Some(secret) = server_config.jwt_secret.as_ref() {
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
        tracing::info!(
            "Using JWKS URL for authentication: {}",
            server_config.jwks_url
        );
        JwtAuthorizer::from_jwks_url(server_config.jwks_url.as_str())
            .validation(validation)
            .check(|claims: &JwtUserClaims| claims.get_username().is_some())
            .build()
            .await
            .whatever_context("Could not create JWT Authorizer from JWKS")
    }
}
