use crate::error::Result;
use futures_util::StreamExt;
use std::{any::Any, collections::HashMap, fmt::Debug};
use tokio::sync::{broadcast::Sender, Mutex};
use tokio_stream::{wrappers::BroadcastStream, Stream};
use tracing::log::{debug, warn};

pub trait Event {
    fn topic() -> &'static str
    where
        Self: Sized;

    fn to_any(self: Box<Self>) -> Box<dyn Any>;

    fn clone_box(&self) -> Box<dyn Event + Send>;
}

impl Clone for Box<dyn Event + Send> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug)]
pub struct EventBus {
    listeners: Mutex<HashMap<&'static str, Sender<Box<dyn Event + Send>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Default::default(),
        }
    }

    pub async fn publish<E: Event + Debug + Send + Clone + 'static>(&self, event: E) -> Result<()> {
        debug!("Event published: {:?}", event);
        let mut listeners = self.listeners.lock().await;
        let tx = listeners
            .entry(E::topic())
            .or_insert_with(Self::insert_listener);
        if let Err(_) = tx.send(Box::new(event)) {
            warn!("No subscribers for topic {}", E::topic());
        }
        Ok(())
    }

    pub async fn subscribe<E>(&self) -> impl Stream<Item = E>
    where
        E: Event + Send + Debug + Clone + 'static,
    {
        debug!("Subscribed to event: {}", E::topic());
        let mut listeners = self.listeners.lock().await;
        let rx = listeners
            .entry(E::topic())
            .or_insert_with(Self::insert_listener)
            .subscribe();
        BroadcastStream::new(rx).map(|event| {
            let event = event
                .expect("Event deserialization failed")
                .to_any()
                .downcast::<E>()
                .expect("Event downcast failed");
            *event
        })
    }

    fn insert_listener() -> Sender<Box<dyn Event + Send>> {
        let (tx, _) = tokio::sync::broadcast::channel::<Box<dyn Event + Send>>(8);
        tx
    }
}
