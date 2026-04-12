use crate::error;
use async_trait::async_trait;

#[async_trait]
pub trait TransactionProvider<Tx>: Send + Sync + 'static {
    async fn begin(&self) -> error::Result<Tx>;
    async fn commit(&self, tx: Tx) -> error::Result<()>;
    async fn rollback(&self, tx: Tx) -> error::Result<()>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub struct MockTx;

    pub struct MockTxProvider {
        pub commits: AtomicUsize,
        pub rollbacks: AtomicUsize,
    }

    impl MockTxProvider {
        pub fn new() -> Self {
            Self {
                commits: AtomicUsize::new(0),
                rollbacks: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl TransactionProvider<MockTx> for MockTxProvider {
        async fn begin(&self) -> error::Result<MockTx> {
            Ok(MockTx)
        }

        async fn commit(&self, _tx: MockTx) -> error::Result<()> {
            self.commits.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn rollback(&self, _tx: MockTx) -> error::Result<()> {
            self.rollbacks.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
}