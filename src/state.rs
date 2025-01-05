use crate::{config::GlobalConfig, error::Result, exif::Exiftool, util::events::EventBus};
use sqlx::{pool::PoolConnection, PgConnection, PgPool, Postgres};
use std::{ops::DerefMut, sync::Arc};
use tokio::sync::{mpsc::Sender, MappedMutexGuard, Mutex, MutexGuard};

pub type Transaction = sqlx::Transaction<'static, Postgres>;

#[derive(Clone)]
pub struct ArcConnection<'e>(Arc<Mutex<&'e mut PgConnection>>);

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

    pub async fn get_connection(&self) -> Result<PoolConnection<Postgres>> {
        let pool_connection = self.db_pool.acquire().await?;
        Ok(pool_connection)
    }
}

impl<'e> ArcConnection<'e> {
    pub fn new(transaction: &'e mut PgConnection) -> Self {
        Self(Arc::new(Mutex::new(transaction)))
    }

    pub async fn get_connection(&self) -> MappedMutexGuard<PgConnection> {
        MutexGuard::map(self.0.lock().await, |guard| guard.deref_mut())
    }
}
