use crate::bus::subscription::SubscriptionOptions;
use crate::bus::EventBus;
use crate::error;
use crate::event::domain_event::DomainEvent;
use async_channel::Receiver;
use async_trait::async_trait;
use futures_util::StreamExt;
use snafu::{ResultExt, Whatever};
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    future::Future,
    sync::Arc,
};

pub type SharedEvent = Arc<dyn DomainEvent>;
use tokio::task::JoinHandle;
use tokio::{
    sync::{broadcast, Mutex},
    task,
};
use tokio_stream::{wrappers::BroadcastStream, Stream};
use tracing::{debug, error, info, warn};

pub struct InMemEventBus {
    listeners: Mutex<HashMap<TypeId, broadcast::Sender<SharedEvent>>>,
}

impl InMemEventBus {
    pub fn new() -> Self {
        Self {
            listeners: Default::default(),
        }
    }
}

#[async_trait]
impl EventBus for InMemEventBus {
    async fn publish(&self, event: impl DomainEvent) -> error::Result<()> {
        let shared: SharedEvent = Arc::new(event);
        self.publish_shared(shared).await
    }

    async fn subscribe<E>(&self) -> impl Stream<Item = Arc<E>>
    where
        E: DomainEvent,
    {
        self.subscribe().await
    }

    async fn start_consumer_with_options<E, F, Fut>(
        &self,
        options: SubscriptionOptions,
        consumer: F,
    ) -> Result<Vec<JoinHandle<()>>, Whatever>
    where
        E: DomainEvent,
        F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
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
            let (work_rx, fanout_handle) = self.start_fanout_consumer(&name).await?;

            let mut handles = vec![fanout_handle];
            handles.extend(Self::start_workers(name, workers, work_rx, consumer)?);
            Ok(handles)
        }
    }
}

impl InMemEventBus {
    pub async fn publish_shared(&self, shared: SharedEvent) -> error::Result<()> {
        let type_id = (*shared).type_id();
        debug!("Event published: {:?}", type_id);
        let mut listeners = self.listeners.lock().await;
        let tx = listeners
            .entry(type_id)
            .or_insert_with(Self::insert_listener);
        if tx.send(shared).is_err() {
            warn!("No subscribers for event type {:?}", type_id);
        }
        Ok(())
    }

    pub async fn subscribe<E>(&self) -> impl Stream<Item = Arc<E>>
    where
        E: DomainEvent,
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
            let any: Arc<dyn Any + Send + Sync> = shared;
            Arc::downcast::<E>(any).expect("Event downcast failed")
        })
    }

    async fn start_consumer<E, F, Fut>(
        &self,
        name: String,
        consumer: F,
    ) -> Result<task::JoinHandle<()>, Whatever>
    where
        E: DomainEvent,
        F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
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
        work_rx: Receiver<Arc<E>>,
        consumer: F,
    ) -> Result<Vec<JoinHandle<()>>, Whatever>
    where
        E: DomainEvent,
        F: Fn(Arc<E>) -> Fut + Send + Sync + Clone + 'static,
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

    async fn start_fanout_consumer<E>(
        &self,
        name: &String,
    ) -> Result<(Receiver<Arc<E>>, JoinHandle<()>), Whatever>
    where
        E: DomainEvent,
    {
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
        Ok((work_rx, fanout_handle))
    }

    fn insert_listener() -> broadcast::Sender<SharedEvent> {
        let (tx, _) = broadcast::channel::<SharedEvent>(32);
        tx
    }
}

impl Default for InMemEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::domain_event::fixtures::{OtherEvent, TestEvent};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn publish_and_subscribe_receives_event() {
        let bus = InMemEventBus::new();
        let mut stream = bus.subscribe::<TestEvent>().await;

        bus.publish(TestEvent::new("hello")).await.unwrap();

        let event = timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(event.value, "hello");
    }

    #[tokio::test]
    async fn multiple_subscribers_each_receive_event() {
        let bus = InMemEventBus::new();
        let mut stream1 = bus.subscribe::<TestEvent>().await;
        let mut stream2 = bus.subscribe::<TestEvent>().await;

        bus.publish(TestEvent::new("broadcast")).await.unwrap();

        let e1 = timeout(Duration::from_secs(1), stream1.next())
            .await
            .unwrap()
            .unwrap();
        let e2 = timeout(Duration::from_secs(1), stream2.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(e1.value, "broadcast");
        assert_eq!(e2.value, "broadcast");
    }

    #[tokio::test]
    async fn different_event_types_are_isolated() {
        let bus = InMemEventBus::new();
        let mut test_stream = bus.subscribe::<TestEvent>().await;
        let mut other_stream = bus.subscribe::<OtherEvent>().await;

        bus.publish(TestEvent::new("only-test")).await.unwrap();

        let event = timeout(Duration::from_secs(1), test_stream.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.value, "only-test");

        // OtherEvent stream should not receive anything
        let result = timeout(Duration::from_millis(50), other_stream.next()).await;
        assert!(
            result.is_err(),
            "should have timed out — no OtherEvent published"
        );
    }

    #[tokio::test]
    async fn publish_without_subscribers_does_not_error() {
        let bus = InMemEventBus::new();
        let result = bus.publish(TestEvent::new("nobody listening")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn start_consumer_processes_events() {
        let bus = InMemEventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let options = SubscriptionOptions::new().named("test-consumer");
        let _handles = bus
            .start_consumer_with_options::<TestEvent, _, _>(options, move |_event| {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            })
            .await
            .unwrap();

        bus.publish(TestEvent::new("a")).await.unwrap();
        bus.publish(TestEvent::new("b")).await.unwrap();

        // Give the consumer task time to process
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn start_consumer_with_multiple_workers() {
        let bus = InMemEventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let options = SubscriptionOptions::new()
            .named("multi-worker")
            .with_workers(3);
        let handles = bus
            .start_consumer_with_options::<TestEvent, _, _>(options, move |_event| {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            })
            .await
            .unwrap();

        // 1 fanout + 3 workers = 4 handles
        assert_eq!(handles.len(), 4);

        for i in 0..5 {
            bus.publish(TestEvent::new(&format!("event-{i}")))
                .await
                .unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn multiple_events_arrive_in_order() {
        let bus = InMemEventBus::new();
        let mut stream = bus.subscribe::<TestEvent>().await;

        for i in 0..5 {
            bus.publish(TestEvent::new(&format!("msg-{i}")))
                .await
                .unwrap();
        }

        for i in 0..5 {
            let event = timeout(Duration::from_secs(1), stream.next())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(event.value, format!("msg-{i}"));
        }
    }
}
