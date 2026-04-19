use std::sync::Arc;

use application::{
    error::{ApplicationError, ApplicationResult},
    event_bus::PublishEvent,
};
use async_trait::async_trait;
use domain::event::DomainEvent;
use event_sourcing::bus::{projection::ProjectionEventBus, EventBus};
use sqlx::{Postgres, Transaction};

/// Newtype wrapper around `ProjectionEventBus` to satisfy Rust's orphan rule
/// when implementing the `PublishEvent` trait from the application layer.
pub struct ProjectionEventBusAdapter {
    inner: Arc<ProjectionEventBus<i64, Transaction<'static, Postgres>>>,
}

impl ProjectionEventBusAdapter {
    pub fn new(inner: Arc<ProjectionEventBus<i64, Transaction<'static, Postgres>>>) -> Self {
        Self { inner }
    }

    /// Access the underlying bus (e.g. for registering projections or consumers).
    pub fn inner(&self) -> &Arc<ProjectionEventBus<i64, Transaction<'static, Postgres>>> {
        &self.inner
    }
}

#[async_trait]
impl<E: DomainEvent + 'static> PublishEvent<E> for ProjectionEventBusAdapter {
    async fn publish(&self, event: E) -> ApplicationResult<()> {
        EventBus::publish(self.inner.as_ref(), event)
            .await
            .map_err(|e| ApplicationError::Internal {
                message: e.to_string(),
            })
    }
}
