#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub client_id: String,
    pub token_url: String,
    pub authorize_url: String,
}
