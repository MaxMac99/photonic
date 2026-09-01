use std::net::IpAddr;

use confique::Config;
use snafu::{OptionExt, Whatever};

#[derive(Debug, Config)]
pub struct ServerConfig {
    #[config(default = "0.0.0.0", env = "HOST")]
    pub host: IpAddr,
    #[config(default = 8080, env = "PORT")]
    pub port: u16,
    #[config(env = "OAUTH_CLIENT_ID")]
    pub client_id: Option<String>,
    #[config(env = "OAUTH_JWKS_URL")]
    pub jwks_url: Option<String>,
    #[config(env = "OAUTH_TOKEN_URL")]
    pub token_url: Option<String>,
    #[config(env = "OAUTH_AUTHORIZE_URL")]
    pub authorize_url: Option<String>,
    #[config(env = "JWT_SECRET")]
    pub jwt_secret: Option<String>,
    /// Set when running more than one instance of the server: selects the
    /// coordination-safe checkpoint store so replicas claim projection
    /// checkpoints instead of double-processing events (ADR 0006).
    #[config(default = false, env = "SERVER_MULTI_INSTANCE")]
    pub multi_instance: bool,
}

/// The OIDC provider settings as a single unit, only available when OIDC is
/// fully configured (see [`ServerConfig::oidc`]).
#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub client_id: String,
    pub jwks_url: String,
    pub token_url: String,
    pub authorize_url: String,
}

impl ServerConfig {
    /// Returns the OIDC provider settings when OIDC is enabled, i.e. when all
    /// OAuth settings are configured. Returns `None` when OIDC is disabled.
    pub fn oidc(&self) -> Option<OidcConfig> {
        Some(OidcConfig {
            client_id: self.client_id.as_ref()?.clone(),
            jwks_url: self.jwks_url.as_ref()?.clone(),
            token_url: self.token_url.as_ref()?.clone(),
            authorize_url: self.authorize_url.as_ref()?.clone(),
        })
    }

    /// Rejects partially configured OIDC setups: a half-set OAuth block would
    /// otherwise silently disable authentication instead of failing loudly.
    pub fn validate_oidc(&self) -> Result<(), Whatever> {
        let configured = [
            self.client_id.is_some(),
            self.jwks_url.is_some(),
            self.token_url.is_some(),
            self.authorize_url.is_some(),
        ];
        if configured.iter().any(|&set| set) && !configured.iter().all(|&set| set) {
            None.whatever_context("OIDC is only partially configured: set all of OAUTH_CLIENT_ID, OAUTH_JWKS_URL, OAUTH_TOKEN_URL and OAUTH_AUTHORIZE_URL, or none of them")
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(
        client_id: Option<String>,
        jwks_url: Option<String>,
        token_url: Option<String>,
        authorize_url: Option<String>,
    ) -> ServerConfig {
        ServerConfig {
            host: "127.0.0.1".parse().unwrap(),
            port: 0,
            client_id,
            jwks_url,
            token_url,
            authorize_url,
            jwt_secret: None,
            multi_instance: false,
        }
    }

    #[test]
    fn oidc_disabled_when_nothing_set() {
        let config = config(None, None, None, None);
        assert!(config.oidc().is_none());
        config.validate_oidc().unwrap();
    }

    #[test]
    fn oidc_enabled_when_fully_configured() {
        let config = config(
            Some("client".into()),
            Some("https://example.com/jwks".into()),
            Some("https://example.com/token".into()),
            Some("https://example.com/authorize".into()),
        );
        let oidc = config.oidc().unwrap();
        assert_eq!(oidc.client_id, "client");
        assert_eq!(oidc.jwks_url, "https://example.com/jwks");
        config.validate_oidc().unwrap();
    }

    #[test]
    fn partially_configured_oidc_is_rejected() {
        let config = config(
            Some("client".into()),
            None,
            Some("https://example.com/token".into()),
            Some("https://example.com/authorize".into()),
        );
        assert!(config.oidc().is_none());
        assert!(config.validate_oidc().is_err());
    }
}
