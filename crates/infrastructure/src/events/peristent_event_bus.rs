use std::marker::PhantomData;
use std::sync::Arc;

use super::{EventAppender, EventBus};
use crate::events::transactional_event_handler::{
    TransactionProvider, TransactionalEventAppender, TransactionalEventHandler,
};
use application::{error::ApplicationResult, event_bus::PublishEvent};
use async_trait::async_trait;
use domain::event::DomainEvent;
use tokio::task;
use tracing::error;

/// Event bus that persists events before dispatching to the in-memory bus.
///
/// Flow: persist (via EventAppender) → dispatch (via EventBus)
pub struct PersistentEventBus<E, TxP, Tx>
where
    E: DomainEvent + 'static,
    TxP: TransactionProvider<Tx>,
{
    tx_provider: TxP,
    tx_listeners: Vec<Box<dyn TransactionalEventHandler<E, Tx>>>,
    inner: EventBus,
}

impl<E, TxP, Tx> PersistentEventBus<E, TxP, Tx>
where
    E: DomainEvent + 'static,
    Tx: Send + 'static,
    TxP: TransactionProvider<Tx>,
{
    pub fn new(
        tx_provider: TxP,
        event_appender: impl TransactionalEventAppender<E, Tx> + 'static,
        inner: EventBus,
    ) -> Self {
        Self {
            tx_provider,
            tx_listeners: vec![Box::new(event_appender)],
            inner,
        }
    }

    pub fn register(&mut self, handler: impl TransactionalEventHandler<E, Tx> + 'static) {
        self.tx_listeners.push(Box::new(handler));
    }
}

#[async_trait]
impl<E, TxP, Tx> PublishEvent<E> for PersistentEventBus<E, TxP, Tx>
where
    E: DomainEvent + 'static,
    TxP: TransactionProvider<Tx> + Send + Sync,
{
    async fn publish(&self, event: E) -> ApplicationResult<()> {
        let mut tx = self.tx_provider.begin();
        for listener in &self.tx_listeners {
            listener.handle(&event, &mut tx).await?;
        }
        self.tx_provider.commit(tx).await?;

        if let Err(e) = self.inner.publish(event).await {
            error!(error = %e, "Event persisted but failed to dispatch");
        }

        Ok(())
    }
}
