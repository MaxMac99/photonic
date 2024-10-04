use crate::config::StorageWorkerConfig;
use common::{ksqldb::KsqlDb, stream::producer::KafkaProducer};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<StorageWorkerConfig>,
    pub producer: KafkaProducer,
    pub ksql_db: Arc<KsqlDb>,
}
