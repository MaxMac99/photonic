use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    future::Future,
    sync::Arc,
};

use async_channel::Receiver;
use async_trait::async_trait;
use futures_util::StreamExt;
use snafu::{ResultExt, Whatever};
use tokio::{
    sync::{broadcast, Mutex},
    task,
};
use tokio_stream::{wrappers::BroadcastStream, Stream};
use tracing::{debug, error, info, warn};

use crate::{
    application::{
        error::{format_error_with_backtrace, ApplicationResult},
        event_bus::{EventProcessor, PublishEvent},
    },
    domain::event::DomainEvent,
    infrastructure::events::subscription::SubscriptionOptions,
};

// Helper trait for cloneable boxed events
trait CloneableEvent: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneableEvent>;
    fn as_any(&self) -> &dyn Any;
}

impl<T: DomainEvent + 'static> CloneableEvent for T {
    fn clone_box(&self) -> Box<dyn CloneableEvent> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for Box<dyn CloneableEvent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct EventBus {
    listeners: Mutex<HashMap<TypeId, broadcast::Sender<Box<dyn CloneableEvent>>>>,
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
        let mut listeners = self.listeners.lock().await;
        let tx = listeners
            .entry(type_id)
            .or_insert_with(Self::insert_listener);
        if tx.send(Box::new(event)).is_err() {
            warn!("No subscribers for event type {}", type_name::<E>());
        }
        Ok(())
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
        self.subscribe_with_options(options, move |event: E| {
            let processor = processor.clone();
            async move {
                if let Err(e) = processor.process(event).await {
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
        F: Fn(E) -> Fut + Send + Clone + 'static,
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
            let (work_tx, work_rx) = async_channel::bounded::<E>(8);

            let fanout_name = format!("{}-fanout", name);
            let fanout_handle = self
                .start_consumer(fanout_name.clone(), move |event: E| {
                    let tx = work_tx.clone();
                    let fanout_name = fanout_name.clone();
                    async move {
                        if let Err(e) = tx.send(event).await {
                            error!(
                                "Failed to send event from '{}' to workers: {}",
                                fanout_name, e
                            );
                        }
                    }
                })
                .await?;

            let mut handles = vec![fanout_handle];
            handles.extend(Self::start_workers(name, workers, work_rx, consumer)?);
            Ok(handles)
        }
    }

    async fn subscribe<E>(&self) -> impl Stream<Item = E>
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
            let boxed = event.expect("Event receive failed");
            let any_ref = boxed.as_any();
            let event = any_ref.downcast_ref::<E>().expect("Event downcast failed");
            event.clone()
        })
    }

    async fn start_consumer<E, F, Fut>(
        &self,
        name: String,
        consumer: F,
    ) -> Result<task::JoinHandle<()>, Whatever>
    where
        E: DomainEvent + 'static,
        F: Fn(E) -> Fut + Send + Clone + 'static,
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
                    debug!("Received event: {:?}", event);
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
        work_rx: Receiver<E>,
        consumer: F,
    ) -> Result<Vec<task::JoinHandle<()>>, Whatever>
    where
        E: DomainEvent + 'static,
        F: Fn(E) -> Fut + Send + Clone + 'static,
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

    fn insert_listener() -> broadcast::Sender<Box<dyn CloneableEvent>> {
        let (tx, _) = broadcast::channel::<Box<dyn CloneableEvent>>(32);
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
