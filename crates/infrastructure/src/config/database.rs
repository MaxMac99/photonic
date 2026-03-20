use confique::Config;

#[derive(Debug, Config)]
pub struct DatabaseConfig {
    #[config(env = "DATABASE_URL")]
    pub url: String,
    #[config(default = 10, env = "DATABASE_MAX_CONNECTIONS")]
    pub max_connections: u32,
    #[config(default = 1, env = "DATABASE_MIN_CONNECTIONS")]
    pub min_connections: u32,
    #[config(default = 30, env = "DATABASE_CONNECTION_TIMEOUT")]
    pub connection_timeout_seconds: u64,
}
