use std::sync::Arc;

use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::event::event_type::EventType;
use crate::event::stream::StreamExtract;
use crate::persistence::sequence::Sequence;
use crate::persistence::stream_link_store::StreamLinkStore;
use crate::projection::handler::MultiEventProjectionHandler;
use async_trait::async_trait;

/// A projection that writes stream link records when events are published.
///
/// Uses registered [`StreamExtract`] extractors to determine which streams
/// an event belongs to, then persists the links via a [`StreamLinkStore`].
/// Runs as a `MultiEventProjectionHandler` within the `ProjectionEventBus`,
/// atomically with other projections.
pub struct StreamLinkingProjection<Seq, Tx> {
    extractors: Vec<Arc<dyn StreamExtract>>,
    link_store: Arc<dyn StreamLinkStore<Seq, Tx>>,
}

impl<Seq, Tx> StreamLinkingProjection<Seq, Tx> {
    pub fn new(
        extractors: Vec<Arc<dyn StreamExtract>>,
        link_store: Arc<dyn StreamLinkStore<Seq, Tx>>,
    ) -> Self {
        Self {
            extractors,
            link_store,
        }
    }
}

#[async_trait]
impl<Seq, Tx> MultiEventProjectionHandler<Seq, Tx> for StreamLinkingProjection<Seq, Tx>
where
    Seq: Sequence,
    Tx: Send + Sync + 'static,
{
    fn name(&self) -> &str {
        "stream_linker"
    }

    fn event_types(&self) -> Vec<EventType> {
        let mut types = Vec::new();
        for extractor in &self.extractors {
            for et in extractor.event_types() {
                if !types.contains(&et) {
                    types.push(et);
                }
            }
        }
        types
    }

    async fn handle(
        &self,
        event: &dyn DomainEvent,
        sequence: Seq,
        tx: &mut Tx,
    ) -> error::Result<()> {
        for extractor in &self.extractors {
            if let Some(stream_id) = extractor.stream_id(event) {
                self.link_store.link(sequence, &stream_id, tx).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{Aggregate, ApplyEvent};
    use crate::event::event_metadata::EventMetadata;
    use crate::event::stream::StreamDefinition;
    use std::sync::Mutex;

    // -- Test events --

    #[derive(Debug)]
    struct TestEvent {
        metadata: EventMetadata,
        id: String,
    }

    impl DomainEvent for TestEvent {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    #[derive(Debug)]
    struct OtherEvent {
        metadata: EventMetadata,
    }

    impl DomainEvent for OtherEvent {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    // -- Test aggregate --

    #[derive(Default)]
    struct TestAggregate;

    impl Aggregate for TestAggregate {
        type Id = String;
        fn aggregate_type() -> &'static str {
            "Test"
        }
    }

    impl ApplyEvent<TestEvent> for TestAggregate {
        fn apply(&mut self, _event: &TestEvent) {}
    }

    // -- Mock link store --

    struct MockLinkStore {
        links: Mutex<Vec<(i64, String)>>, // (sequence, storage_key)
    }

    impl MockLinkStore {
        fn new() -> Self {
            Self {
                links: Mutex::new(Vec::new()),
            }
        }

        fn link_count(&self) -> usize {
            self.links.lock().unwrap().len()
        }

        fn links(&self) -> Vec<(i64, String)> {
            self.links.lock().unwrap().clone()
        }
    }

    struct MockTx;

    #[async_trait]
    impl StreamLinkStore<i64, MockTx> for MockLinkStore {
        async fn link(
            &self,
            sequence: i64,
            stream: &crate::event::stream::StreamId,
            _tx: &mut MockTx,
        ) -> error::Result<()> {
            self.links
                .lock()
                .unwrap()
                .push((sequence, stream.to_storage_key()));
            Ok(())
        }
    }

    fn test_stream() -> StreamDefinition<TestAggregate> {
        StreamDefinition::<TestAggregate>::builder()
            .with::<TestEvent>(|e| Some(e.id.clone()))
            .build()
    }

    #[tokio::test]
    async fn links_matching_events() {
        let link_store = Arc::new(MockLinkStore::new());
        let stream_def: Arc<StreamDefinition<TestAggregate>> = Arc::new(test_stream());
        let projection =
            StreamLinkingProjection::new(vec![stream_def], link_store.clone());

        let event = TestEvent {
            metadata: EventMetadata::default(),
            id: "abc".to_string(),
        };

        projection.handle(&event, 42, &mut MockTx).await.unwrap();

        let links = link_store.links();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, 42);
        assert_eq!(links[0].1, "Test-abc");
    }

    #[tokio::test]
    async fn skips_non_matching_events() {
        let link_store = Arc::new(MockLinkStore::new());
        let stream_def: Arc<StreamDefinition<TestAggregate>> = Arc::new(test_stream());
        let projection =
            StreamLinkingProjection::new(vec![stream_def], link_store.clone());

        let event = OtherEvent {
            metadata: EventMetadata::default(),
        };

        projection.handle(&event, 42, &mut MockTx).await.unwrap();

        assert_eq!(link_store.link_count(), 0);
    }

    #[test]
    fn event_types_is_union_of_extractors() {
        let stream_def: Arc<StreamDefinition<TestAggregate>> = Arc::new(test_stream());
        let link_store = Arc::new(MockLinkStore::new());
        let projection =
            StreamLinkingProjection::new(vec![stream_def], link_store);

        let types = projection.event_types();
        assert_eq!(types.len(), 1);
    }
}
