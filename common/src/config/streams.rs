use confique::Config;

#[derive(Debug, Config)]
pub struct StreamConfig {
    #[config(default = "http://localhost:8081", env = "SCHEMA_REGISTRY_URL")]
    pub schema_registry_url: String,
    #[config(default = "localhost:9092", env = "BROKER_URL")]
    pub broker_url: String,
    #[config(default = "http://localhost:8087", env = "KSQLDB_URL")]
    pub ksqldb_url: String,
}
