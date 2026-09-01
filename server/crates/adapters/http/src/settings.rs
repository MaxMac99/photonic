/// Auth settings needed by the HTTP adapter, mapped from the global
/// configuration by the composition root.
///
/// All fields are optional: when no authentication is configured the server
/// runs without the authorization layer.
#[derive(Debug, Clone)]
pub struct AuthSettings {
    pub client_id: Option<String>,
    pub jwt_secret: Option<String>,
    pub jwks_url: Option<String>,
}
