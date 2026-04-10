use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::event::event_type::EventType;
use async_trait::async_trait;

/// A projection handler that processes a single event type within a transaction.
#[async_trait]
pub trait ProjectionHandler<E: DomainEvent, Tx>: Send + Sync + 'static {
    /// Unique name for this projection, used as the checkpoint key.
    fn name(&self) -> &str;

    async fn handle(&self, event: &E, tx: &mut Tx) -> error::Result<()>;
}

/// A projection handler that processes multiple event types within a transaction.
#[async_trait]
pub trait MultiEventProjectionHandler<Tx>: Send + Sync + 'static {
    /// Unique name for this projection, used as the checkpoint key.
    fn name(&self) -> &str;

    fn event_types(&self) -> Vec<EventType>;

    async fn handle(&self, event: &dyn DomainEvent, tx: &mut Tx) -> error::Result<()>;
}
