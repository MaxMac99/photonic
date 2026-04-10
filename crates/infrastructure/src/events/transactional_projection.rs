use application::error::ApplicationResult;
use async_trait::async_trait;

#[async_trait]
pub trait TransactionalProjectionStore<Tx> {
    async fn load_checkpoint(&self, projection_name: &str, tx: &mut Tx) -> ApplicationResult<i64>;

    async fn save_checkpoint(
        &self,
        projection_name: &str,
        checkpoint_id: i64,
        tx: &mut Tx,
    ) -> ApplicationResult<()>;

    async fn fetch_events<E>(
        &self,
        after_sequence: i64,
        tx: &mut Tx,
    ) -> ApplicationResult<Vec<E>>
    where
        E: StorableEvent
}
