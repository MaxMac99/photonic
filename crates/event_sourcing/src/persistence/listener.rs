use async_trait::async_trait;
use tokio_stream::Stream;

use crate::{error, event::event_type::EventType};

#[async_trait]
pub trait EventListener<Seq>: Send + Sync + 'static {
    /// Returns a stream of sequence numbers for events matching the given types.
    async fn listen(
        &self,
        event_types: Vec<EventType>,
    ) -> error::Result<impl Stream<Item = Seq> + Send>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use tokio::sync::Mutex;
    use tokio_stream::wrappers::ReceiverStream;

    use super::*;

    pub struct MockListener {
        rx: Mutex<Option<tokio::sync::mpsc::Receiver<()>>>,
    }

    impl MockListener {
        pub fn new() -> (Self, tokio::sync::mpsc::Sender<()>) {
            let (tx, rx) = tokio::sync::mpsc::channel(16);
            (
                Self {
                    rx: Mutex::new(Some(rx)),
                },
                tx,
            )
        }
    }

    #[async_trait]
    impl EventListener<()> for MockListener {
        async fn listen(
            &self,
            _event_types: Vec<EventType>,
        ) -> error::Result<impl Stream<Item = ()> + Send> {
            let rx = self
                .rx
                .lock()
                .await
                .take()
                .expect("listen called more than once");
            Ok(ReceiverStream::new(rx))
        }
    }
}
