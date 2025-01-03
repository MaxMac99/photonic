use crate::{config::GlobalConfig, error::Result, exif::Exiftool, util::events::EventBus};
use sqlx::{PgPool, Postgres};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub type Transaction = sqlx::Transaction<'static, Postgres>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GlobalConfig>,
    pub event_bus: Arc<EventBus>,
    pub died: Sender<bool>,
    pub exiftool: Arc<Exiftool>,
    db_pool: PgPool,
}

impl AppState {
    pub fn new(
        config: Arc<GlobalConfig>,
        event_bus: EventBus,
        db_pool: PgPool,
        exiftool: Exiftool,
        died: Sender<bool>,
    ) -> Self {
        Self {
            config,
            event_bus: Arc::new(event_bus),
            exiftool: Arc::new(exiftool),
            died,
            db_pool,
        }
    }

    pub async fn begin_transaction(&self) -> Result<Transaction> {
        let transaction = self.db_pool.begin().await?;
        Ok(transaction)
    }
}
