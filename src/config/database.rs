use confique::Config;

#[derive(Debug, Config)]
pub struct DatabaseConfig {
    #[config(env = "DATABASE_URL")]
    pub url: String,
}
