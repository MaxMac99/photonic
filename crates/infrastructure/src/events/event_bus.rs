use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    future::Future,
    sync::Arc,
};

use application::{
    error::{format_error_with_backtrace, ApplicationResult},
    event_bus::{EventProcessor, PublishEvent},
};
use async_trait::async_trait;
use domain::event::DomainEvent;
use futures_util::StreamExt;
use snafu::{ResultExt, Whatever};
use tokio::{
    sync::{broadcast, Mutex},
    task,
};
use tokio_stream::{wrappers::BroadcastStream, Stream};
use tracing::{debug, error, info, warn};

use crate::events::subscription::SubscriptionOptions;

/// Shared immutable event wrapper. Arc clone in broadcast is just a ref count increment.
type SharedEvent = Arc<dyn Any + Send + Sync>;

pub struct EventBus {
    listeners: Mutex<HashMap<TypeId, broadcast::Sender<SharedEvent>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Default::default(),
        }
    }

    pub async fn publish<E: DomainEvent + 'static>(&self, event: E) -> ApplicationResult<()> {
        debug!("Event published: {}", type_name::<E>());
        let type_id = TypeId::of::<E>();
        let shared: SharedEvent = Arc::new(event);
        let mut listeners = self.listeners.lock().await;
        let tx = listeners
            .entry(type_id)
            .or_insert_with(Self::insert_listener);
        if tx.send(shared).is_err() {
            warn!("No subscribers for event type {}", type_name::<E>());
        }
        Ok(())
    }

    /// Subscribe to events of type E. Returns a stream of Arc-wrapped events.
    /// Consumers receive &E via downcast_ref — zero copy from publish to consume.
    pub async fn subscribe<E>(&self) -> impl Stream<Item = Arc<E>>
    where
        E: DomainEvent + 'static,
    {
        debug!("Subscribed to event: {}", type_name::<E>());
        let type_id = TypeId::of::<E>();
        let mut listeners = self.listeners.lock().await;
        let rx = listeners
            .entry(type_id)
            .or_insert_with(Self::insert_listener)
            .subscribe();
        BroadcastStream::new(rx).map(|event| {
            let shared = event.expect("Event receive failed");
            Arc::downcast::<E>(shared).expect("Event downcast failed")
        })
    }

    pub async fn start_processor<E, P>(
        &self,
        processor: P,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent + 'static,
        P: EventProcessor<E> + 'static,
    {
        let options = SubscriptionOptions {
            workers: 1,
            name: Some(type_name::<P>().to_string()),
        };
        self.start_processor_with_options(options, processor).await
    }

    pub async fn start_processor_with_options<E, P>(
        &self,
        options: SubscriptionOptions,
        processor: P,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent + 'static,
        P: EventProcessor<E> + 'static,
    {
        let processor = Arc::new(processor);
        self.subscribe_with_options(options, move |event: Arc<E>| {
            let processor = processor.clone();
            async move {
                if let Err(e) = processor.process(&event).await {
                    error!(error = %format_error_with_backtrace(&e), "Failed to process event in processor '{}'",
                        type_name::<P>()
                    );
                }
            }
        })
        .await
    }

    pub async fn subscribe_with_options<E, F, Fut>(
        &self,
        options: SubscriptionOptions,
        consumer: F,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent + 'static,
        F: Fn(Arc<E>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let name = options
            .name
            .unwrap_or_else(|| String::from(type_name::<E>()));
        let workers = options.workers;

        if workers == 1 {
            let handle = self.start_consumer(name, consumer).await?;
            Ok(vec![handle])
        } else {
            // Multi-worker: fanout sends SharedEvent (Arc) — no cloning of event data
            let (work_tx, work_rx) = async_channel::bounded::<Arc<E>>(8);

            let mut stream = self.subscribe::<E>().await;
            let fanout_name = format!("{}-fanout", name);
            let fanout_task_name = fanout_name.clone();
            let fanout_handle = task::Builder::new()
                .name(&fanout_task_name)
                .spawn(async move {
                    info!(
                        "Fanout '{}' started for topic '{}'",
                        fanout_name,
                        type_name::<E>()
                    );
                    while let Some(shared) = stream.next().await {
                        if let Err(e) = work_tx.send(shared).await {
                            error!("Failed to send from '{}' to workers: {}", fanout_name, e);
                        }
                    }
                })
                .whatever_context("Failed to spawn fanout task")?;

            let mut handles = vec![fanout_handle];
            handles.extend(Self::start_workers(name, workers, work_rx, consumer)?);
            Ok(handles)
        }
    }

    async fn start_consumer<E, F, Fut>(
        &self,
        name: String,
        consumer: F,
    ) -> Result<task::JoinHandle<()>, Whatever>
    where
        E: DomainEvent + 'static,
        F: Fn(Arc<E>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut stream = self.subscribe::<E>().await;
        let task_name = name.clone();
        let handle = task::Builder::new()
            .name(&task_name)
            .spawn(async move {
                info!(
                    "Started consumer task '{}' for topic '{}'",
                    name,
                    type_name::<E>()
                );
                while let Some(event) = stream.next().await {
                    consumer(event).await;
                }
                info!("Consumer task '{}' shutting down", name);
            })
            .whatever_context("Failed to spawn consumer task")?;
        Ok(handle)
    }

    fn start_workers<E, F, Fut>(
        name: String,
        workers: usize,
        work_rx: async_channel::Receiver<Arc<E>>,
        consumer: F,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent + 'static,
        F: Fn(Arc<E>) -> Fut + Send + Clone + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut handles = Vec::with_capacity(workers);
        for worker_id in 0..workers {
            let work_rx = work_rx.clone();
            let worker_name = format!("{}-worker-{}", name, worker_id);
            let consumer = consumer.clone();
            let task_name = worker_name.clone();

            let handle = task::Builder::new()
                .name(&task_name)
                .spawn(async move {
                    info!(
                        "Worker '{}' started for topic '{}'",
                        worker_name,
                        type_name::<E>()
                    );

                    while let Ok(event) = work_rx.recv().await {
                        consumer(event).await;
                    }

                    info!("Worker '{}' stopped", worker_name);
                })
                .whatever_context("Failed to spawn worker task")?;
            handles.push(handle);
        }
        Ok(handles)
    }

    fn insert_listener() -> broadcast::Sender<SharedEvent> {
        let (tx, _) = broadcast::channel::<SharedEvent>(32);
        tx
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E: DomainEvent + 'static> PublishEvent<E> for EventBus {
    async fn publish(&self, event: E) -> ApplicationResult<()> {
        self.publish(event).await
    }
}
