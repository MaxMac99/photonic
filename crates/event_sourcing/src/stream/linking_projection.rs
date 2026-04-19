use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    error,
    event::domain_event::DomainEvent,
    persistence::sequence::Sequence,
    projection::handler::CatchAllProjectionHandler,
    stream::{definition::StreamExtract, link_store::StreamLinkStore},
};

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
impl<Seq, Tx> CatchAllProjectionHandler<Seq, Tx> for StreamLinkingProjection<Seq, Tx>
where
    Seq: Sequence,
    Tx: Send + Sync + 'static,
{
    type Error = error::EventSourcingError;

    fn name(&self) -> &str {
        "stream_linker"
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
    use crate::{
        aggregate::traits::{Aggregate, ApplyEvent},
        event::event_metadata::EventMetadata,
        stream::{definition::StreamDefinition, link_store::fixtures::MockStreamLinkStore},
    };

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

    struct MockTx;

    fn test_stream() -> StreamDefinition<TestAggregate> {
        StreamDefinition::<TestAggregate>::builder()
            .with::<TestEvent>(|e| Some(e.id.clone()))
            .build()
    }

    #[tokio::test]
    async fn links_matching_events() {
        let link_store = Arc::new(MockStreamLinkStore::new());
        let stream_def: Arc<StreamDefinition<TestAggregate>> = Arc::new(test_stream());
        let projection = StreamLinkingProjection::new(vec![stream_def], link_store.clone());

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
        let link_store = Arc::new(MockStreamLinkStore::new());
        let stream_def: Arc<StreamDefinition<TestAggregate>> = Arc::new(test_stream());
        let projection = StreamLinkingProjection::new(vec![stream_def], link_store.clone());

        let event = OtherEvent {
            metadata: EventMetadata::default(),
        };

        projection.handle(&event, 42, &mut MockTx).await.unwrap();

        assert_eq!(link_store.link_count(), 0);
    }
}
