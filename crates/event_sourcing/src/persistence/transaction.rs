use crate::error;
use async_trait::async_trait;

#[async_trait]
pub trait TransactionProvider<Tx>: Send + Sync + 'static {
    async fn begin(&self) -> error::Result<Tx>;
    async fn commit(&self, tx: Tx) -> error::Result<()>;
    async fn rollback(&self, tx: Tx) -> error::Result<()>;
}