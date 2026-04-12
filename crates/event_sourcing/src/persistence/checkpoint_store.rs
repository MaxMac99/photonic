use crate::error;
use async_trait::async_trait;

/// Persists the last-processed sequence per named consumer.
///
/// On startup, consumers load their checkpoint to resume from where
/// they left off. After processing each event, they save the new checkpoint.
#[async_trait]
pub trait CheckpointStore<Seq>: Send + Sync + 'static {
    /// Load the last committed sequence for this consumer.
    /// Returns `Seq::default()` if no checkpoint exists.
    async fn load(&self, consumer_name: &str) -> error::Result<Seq>;

    /// Save the checkpoint for this consumer.
    async fn save(&self, consumer_name: &str, sequence: Seq) -> error::Result<()>;
}

/// Transactional variant of [`CheckpointStore`]. Saves checkpoints within
/// an existing transaction so the checkpoint update is atomic with the
/// consumer's work (e.g. projection read model updates).
#[async_trait]
pub trait TxCheckpointStore<Seq, Tx>: Send + Sync + 'static {
    async fn load(&self, consumer_name: &str, tx: &mut Tx) -> error::Result<Seq>;
    async fn save(&self, consumer_name: &str, sequence: Seq, tx: &mut Tx) -> error::Result<()>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use std::collections::HashMap;

    pub struct MockCheckpointStore {
        pub checkpoints: std::sync::Mutex<HashMap<String, i64>>,
    }

    impl MockCheckpointStore {
        pub fn new() -> Self {
            Self {
                checkpoints: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CheckpointStore<i64> for MockCheckpointStore {
        async fn load(&self, consumer_name: &str) -> error::Result<i64> {
            Ok(*self
                .checkpoints
                .lock()
                .unwrap()
                .get(consumer_name)
                .unwrap_or(&0))
        }

        async fn save(&self, consumer_name: &str, sequence: i64) -> error::Result<()> {
            self.checkpoints
                .lock()
                .unwrap()
                .insert(consumer_name.to_string(), sequence);
            Ok(())
        }
    }

    pub struct MockTxCheckpointStore {
        pub checkpoints: std::sync::Mutex<HashMap<String, i64>>,
    }

    impl MockTxCheckpointStore {
        pub fn new() -> Self {
            Self {
                checkpoints: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl<Tx: Send + Sync + 'static> TxCheckpointStore<i64, Tx> for MockTxCheckpointStore {
        async fn load(&self, consumer_name: &str, _tx: &mut Tx) -> error::Result<i64> {
            Ok(*self
                .checkpoints
                .lock()
                .unwrap()
                .get(consumer_name)
                .unwrap_or(&0))
        }

        async fn save(
            &self,
            consumer_name: &str,
            sequence: i64,
            _tx: &mut Tx,
        ) -> error::Result<()> {
            self.checkpoints
                .lock()
                .unwrap()
                .insert(consumer_name.to_string(), sequence);
            Ok(())
        }
    }
}