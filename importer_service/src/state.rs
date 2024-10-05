use crate::config::ImporterWorkerConfig;
use common::{ksqldb::KsqlDb, stream::producer::KafkaProducer};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ImporterWorkerConfig>,
    pub producer: KafkaProducer,
    pub ksql_db: Arc<KsqlDb>,
}
