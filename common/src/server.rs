use crate::{config::ServerConfig, user::User};
use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use snafu::{ResultExt, Whatever};
use tokio::{signal, sync::oneshot::Receiver};
use tracing::log::debug;

pub async fn shutdown_signal_with_external_signal(died_receiver: Receiver<bool>) {
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
        _ = died_receiver => {},
    }

    debug!("signal received, starting graceful shutdown");
}

pub async fn shutdown_signal() {
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
    }

    debug!("signal received, starting graceful shutdown");
}

pub async fn setup_auth(server_config: &ServerConfig) -> Result<Authorizer<User>, Whatever> {
    let validation = Validation::new().aud(&[server_config.client_id.clone()]);
    JwtAuthorizer::from_jwks_url(server_config.jwks_url.as_str())
        .validation(validation)
        .check(|user: &User| {
            user.given_name.is_some()
                || user.name.is_some()
                || user.nickname.is_some()
                || user.preferred_username.is_some()
        })
        .build()
        .await
        .whatever_context("Could not create JWT Authorizer")
}