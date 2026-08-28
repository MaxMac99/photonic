use async_trait::async_trait;

use crate::{app_error::ApplicationResult, event::DomainEvent};

#[async_trait]
pub trait PublishEvent<E: DomainEvent>: Send + Sync {
    async fn publish(&self, event: E) -> ApplicationResult<()>;
}

// Re-export from event_sourcing — application listeners implement this directly
pub use event_sourcing::bus::EventProcessor;
