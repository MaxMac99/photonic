mod event_processor;
pub mod inmem;
pub mod notifying_persistent;
pub mod persistent;
pub mod projection;
pub mod subscription;

use crate::bus::subscription::SubscriptionOptions;
use crate::error;
use crate::event::domain_event::DomainEvent;
use async_trait::async_trait;
pub use event_processor::EventProcessor;
use snafu::Whatever;
use std::any::type_name;
use std::future::Future;
use std::sync::Arc;
use tokio::task;
use tokio_stream::Stream;
use tracing::error;

#[async_trait]
pub trait EventBus {
    async fn publish(&self, event: impl DomainEvent) -> error::Result<()>;

    async fn subscribe<E>(&self) -> impl Stream<Item = Arc<E>>
    where
        E: DomainEvent;

    async fn start_consumer_with_options<E, F, Fut>(
        &self,
        options: SubscriptionOptions,
        consumer: F,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent,
        F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static;

    async fn start_processor<E, P>(
        &self,
        processor: P,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent,
        P: EventProcessor<E>,
    {
        let options = SubscriptionOptions::new().named(type_name::<P>());
        self.start_processor_with_options(options, processor).await
    }

    async fn start_processor_with_options<E, P>(
        &self,
        options: SubscriptionOptions,
        processor: P,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent,
        P: EventProcessor<E>,
    {
        let processor = Arc::new(processor);
        self.start_consumer_with_options(options, move |event: Arc<E>| {
            let processor = processor.clone();
            async move {
                if let Err(e) = processor.process(&event).await {
                    error!(error = %e, "Failed to process event in processor '{}'",
                        type_name::<P>()
                    );
                }
            }
        })
        .await
    }
}
