use application::error::ApplicationResult;
use async_trait::async_trait;
use domain::event::DomainEvent;

/// A handler that processes an event within a shared transaction.
/// All handlers for the same event share the same `Tx`, ensuring atomicity.
///
/// `Tx` is opaque to the application layer — the infrastructure decides
/// what it is (e.g., PgTransaction, in-memory fake).
#[async_trait]
pub trait TransactionalEventHandler<E: DomainEvent, Tx: Send>: Send + Sync {
    async fn handle(&self, event: &E, tx: &mut Tx) -> ApplicationResult<()>;
}

#[async_trait]
pub trait TransactionProvider<Tx> {
    async fn begin(&self) -> ApplicationResult<Tx>;

    async fn commit(&self, tx: Tx) -> ApplicationResult<()>;
}

#[async_trait]
pub trait TransactionalEventAppender<E: DomainEvent, Tx>: Send + Sync {
    async fn append(&self, event: &E, tx: &mut Tx) -> ApplicationResult<()>;
}

/// A TransactionalEventAppender automatically works as a TransactionalEventHandler.
#[async_trait]
impl<E, Tx, A> TransactionalEventHandler<E, Tx> for A
where
    E: DomainEvent,
    Tx: Send,
    A: TransactionalEventAppender<E, Tx>,
{
    async fn handle(&self, event: &E, tx: &mut Tx) -> ApplicationResult<()> {
        self.append(event, tx).await
    }
}
