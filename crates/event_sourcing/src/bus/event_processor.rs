use std::sync::Arc;

use async_trait::async_trait;

use crate::{error::Result, event::domain_event::DomainEvent};

#[async_trait]
pub trait EventProcessor<E: DomainEvent>: Send + Sync + 'static {
    async fn process(&self, event: &E) -> Result<()>;
}

#[async_trait]
impl<E, T> EventProcessor<E> for Arc<T>
where
    E: DomainEvent,
    T: EventProcessor<E>,
{
    async fn process(&self, event: &E) -> Result<()> {
        T::process(self, event).await
    }
}
