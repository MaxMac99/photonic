use std::borrow::Cow;

use async_trait::async_trait;
use domain::event::DomainEvent;

use crate::error::ApplicationResult;

/// A projection consumes event events and updates a read model.
///
/// Projections are guaranteed to run before other event listeners,
/// ensuring read models are current when listeners query them.
///
/// The same projection handles both live events (direct from EventBus)
/// and replayed events (from the event store during catch-up).
#[async_trait]
pub trait Projection<E: DomainEvent>: Send + Sync {
    /// Unique name for checkpoint tracking
    fn name(&self) -> Cow<'static, str>;

    /// Process a single event and update the read model
    async fn handle(&self, event: &E, global_sequence: i64) -> ApplicationResult<()>;
}
