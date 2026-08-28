use byte_unit::Byte;

#[derive(Debug, Clone)]
pub struct QuotaConfig {
    pub default_user_quota: Byte,
    pub max_user_quota: Byte,
}
