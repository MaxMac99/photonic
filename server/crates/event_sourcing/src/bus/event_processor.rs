use std::sync::Arc;

use async_trait::async_trait;

use crate::event::domain_event::DomainEvent;

#[async_trait]
pub trait EventProcessor<E: DomainEvent>: Send + Sync + 'static {
    type Error: std::fmt::Display + Send;

    async fn process(&self, event: &E) -> Result<(), Self::Error>;
}

#[async_trait]
impl<E, T> EventProcessor<E> for Arc<T>
where
    E: DomainEvent,
    T: EventProcessor<E>,
{
    type Error = T::Error;

    async fn process(&self, event: &E) -> Result<(), Self::Error> {
        T::process(self, event).await
    }
}
