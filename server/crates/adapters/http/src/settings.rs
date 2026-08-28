/// Auth settings needed by the HTTP adapter, mapped from the global
/// configuration by the composition root.
#[derive(Debug, Clone)]
pub struct AuthSettings {
    pub client_id: String,
    pub jwt_secret: Option<String>,
    pub jwks_url: String,
}
