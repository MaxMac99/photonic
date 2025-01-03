use confique::Config;
use std::net::IpAddr;

#[derive(Debug, Config)]
pub struct ServerConfig {
    #[config(default = "0.0.0.0", env = "HOST")]
    pub host: IpAddr,
    #[config(default = 8080, env = "PORT")]
    pub port: u16,
    #[config(env = "OAUTH_CLIENT_ID")]
    pub client_id: String,
    #[config(env = "OAUTH_JWKS_URL")]
    pub jwks_url: String,
}
