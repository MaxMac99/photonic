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