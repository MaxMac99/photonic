mod event_bus;
mod eventual_consistent_event_bus;
mod peristent_event_bus;
mod transactional_event_handler;
pub mod transactional_projection;

use application::error::ApplicationResult;
use async_trait::async_trait;
use domain::event::DomainEvent;
pub use event_bus::EventBus;
pub use eventual_consistent_event_bus::EventualConsistentEventBus;
pub use peristent_event_bus::PersistentEventBus;
pub use transactional_event_handler::*;

/// Port for persisting a single event.
/// Used by PersistentEventBus to store events before dispatching.
#[async_trait]
pub trait EventAppender<E: DomainEvent>: Send + Sync {
    async fn append(&self, event: &E) -> ApplicationResult<()>;
}
