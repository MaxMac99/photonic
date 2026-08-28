use async_trait::async_trait;
use event_sourcing::{
    error::{EventSourcingError, Result},
    persistence::transaction::TransactionProvider,
};
use sqlx::{PgPool, Postgres, Transaction};

pub struct PostgresTransactionProvider {
    pool: PgPool,
}

impl PostgresTransactionProvider {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionProvider<Transaction<'static, Postgres>> for PostgresTransactionProvider {
    async fn begin(&self) -> Result<Transaction<'static, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| EventSourcingError::Transaction {
                message: format!("Failed to begin transaction: {e}"),
            })
    }

    async fn commit(&self, tx: Transaction<'static, Postgres>) -> Result<()> {
        tx.commit()
            .await
            .map_err(|e| EventSourcingError::Transaction {
                message: format!("Failed to commit transaction: {e}"),
            })
    }

    async fn rollback(&self, tx: Transaction<'static, Postgres>) -> Result<()> {
        tx.rollback()
            .await
            .map_err(|e| EventSourcingError::Transaction {
                message: format!("Failed to rollback transaction: {e}"),
            })
    }
}
