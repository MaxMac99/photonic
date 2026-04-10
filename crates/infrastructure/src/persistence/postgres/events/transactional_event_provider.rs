use application::error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};

use crate::events::TransactionProvider;

#[async_trait]
impl TransactionProvider<Transaction<'static, Postgres>> for PgPool {
    async fn begin(&self) -> ApplicationResult<Transaction<'static, Postgres>> {
        self.begin().await.map_err(|e| ApplicationError::Internal {
            message: format!("Failed to begin transaction: {}", e),
        })
    }

    async fn commit(&self, tx: Transaction<'static, Postgres>) -> ApplicationResult<()> {
        tx.commit().await.map_err(|e| ApplicationError::Internal {
            message: format!("Failed to commit transaction: {}", e),
        })
    }
}
