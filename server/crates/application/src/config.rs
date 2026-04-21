use byte_unit::Byte;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub client_id: String,
    pub token_url: String,
    pub authorize_url: String,
}

#[derive(Debug, Clone)]
pub struct QuotaConfig {
    pub default_user_quota: Byte,
    pub max_user_quota: Byte,
}
