use async_trait::async_trait;

use crate::{error, stream::stream_id::StreamId};

/// Writes stream link records within a transaction.
///
/// When an event is published, the [`StreamLinkingProjection`](crate::stream::linking_projection::StreamLinkingProjection)
/// determines which streams the event belongs to and calls this store to
/// persist the links. The implementation assigns per-stream versions
/// internally (e.g. via `MAX(version) + 1` or a database sequence).
#[async_trait]
pub trait StreamLinkStore<Seq, Tx>: Send + Sync + 'static {
    /// Link an event (by global sequence) to a stream within a transaction.
    async fn link(&self, sequence: Seq, stream: &StreamId, tx: &mut Tx) -> error::Result<()>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::sync::Mutex;

    use super::*;

    pub struct MockStreamLinkStore {
        links: Mutex<Vec<(i64, String)>>,
    }

    impl MockStreamLinkStore {
        pub fn new() -> Self {
            Self {
                links: Mutex::new(Vec::new()),
            }
        }

        pub fn link_count(&self) -> usize {
            self.links.lock().unwrap().len()
        }

        pub fn links(&self) -> Vec<(i64, String)> {
            self.links.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl<Tx: Send + Sync + 'static> StreamLinkStore<i64, Tx> for MockStreamLinkStore {
        async fn link(&self, sequence: i64, stream: &StreamId, _tx: &mut Tx) -> error::Result<()> {
            self.links
                .lock()
                .unwrap()
                .push((sequence, stream.to_storage_key()));
            Ok(())
        }
    }
}
