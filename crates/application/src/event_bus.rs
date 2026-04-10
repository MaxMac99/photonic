use std::sync::Arc;

use async_trait::async_trait;
use domain::event::DomainEvent;

use crate::error::ApplicationResult;

#[async_trait]
pub trait PublishEvent<E: DomainEvent>: Send + Sync {
    async fn publish(&self, event: E) -> ApplicationResult<()>;
}

#[async_trait]
pub trait EventProcessor<E: DomainEvent>: Send + Sync {
    async fn process(&self, event: &E) -> ApplicationResult<()>;
}

#[async_trait]
impl<E, T> EventProcessor<E> for Arc<T>
where
    E: DomainEvent + 'static,
    T: EventProcessor<E> + ?Sized,
{
    async fn process(&self, event: &E) -> ApplicationResult<()> {
        T::process(self, event).await
    }
}
