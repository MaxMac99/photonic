use async_trait::async_trait;
use domain::event::DomainEvent;

use crate::error::ApplicationResult;

#[async_trait]
pub trait PublishEvent<E: DomainEvent>: Send + Sync {
    async fn publish(&self, event: E) -> ApplicationResult<()>;
}

// Re-export from event_sourcing — application listeners implement this directly
pub use event_sourcing::bus::EventProcessor;
