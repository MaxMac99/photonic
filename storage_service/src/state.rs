use crate::config::StorageWorkerConfig;
use common::stream::producer::KafkaProducer;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<StorageWorkerConfig>,
    pub producer: KafkaProducer,
    pub db_pool: PgPool,
}
