use std::sync::Arc;

use application::{error::ApplicationResult, event_bus::PublishEvent};
use async_trait::async_trait;
use domain::event::DomainEvent;
use tracing::error;

use super::{EventAppender, EventBus};

/// Event bus that persists events before dispatching to the in-memory bus.
///
/// Flow: persist (via EventAppender) → dispatch (via EventBus)
pub struct EventualConsistentEventBus<A> {
    appender: Arc<A>,
    inner: Arc<EventBus>,
}

impl<A> EventualConsistentEventBus<A> {
    pub fn new(appender: Arc<A>, inner: Arc<EventBus>) -> Self {
        Self { appender, inner }
    }
}

#[async_trait]
impl<A, E> PublishEvent<E> for EventualConsistentEventBus<A>
where
    A: EventAppender<E> + Send + Sync + 'static,
    E: DomainEvent + 'static,
{
    async fn publish(&self, event: E) -> ApplicationResult<()> {
        self.appender.append(&event).await?;

        if let Err(e) = self.inner.publish(event).await {
            error!(error = %e, "Event persisted but failed to dispatch");
        }

        Ok(())
    }
}
