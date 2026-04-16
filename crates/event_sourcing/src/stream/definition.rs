use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::{
    aggregate::traits::{Aggregate, AggregateType, ApplyEvent},
    event::domain_event::DomainEvent,
    persistence::event_store::StoredEvent,
    stream::stream_id::StreamId,
};

type StreamIdExtractor = Arc<dyn Fn(&dyn DomainEvent) -> Option<String> + Send + Sync>;
type EventApplier<A> = Box<dyn Fn(&mut A, &dyn DomainEvent) + Send + Sync>;

/// A typed stream definition that knows:
/// - Which event types belong to this stream
/// - How to extract a stream id from each event type
/// - How to apply each event type to the aggregate (for reconstitution)
///
/// Built via [`StreamDefinitionBuilder`]. The builder enforces at compile
/// time that the aggregate implements [`ApplyEvent<E>`] for every registered
/// event type.
pub struct StreamDefinition<A> {
    aggregate_type: AggregateType,
    extractors: HashMap<TypeId, StreamIdExtractor>,
    appliers: HashMap<TypeId, EventApplier<A>>,
}

impl<A: Aggregate> StreamDefinition<A> {
    pub fn builder() -> StreamDefinitionBuilder<A> {
        StreamDefinitionBuilder {
            aggregate_type: AggregateType::of::<A>(),
            extractors: HashMap::new(),
            appliers: HashMap::new(),
        }
    }

    /// Returns the `StreamId` for an event if it belongs to this stream.
    pub fn stream_id(&self, event: &dyn DomainEvent) -> Option<StreamId> {
        let type_id = (*event).type_id();
        let extractor = self.extractors.get(&type_id)?;
        extractor(event).map(|id| StreamId::new(self.aggregate_type.clone(), id))
    }

    /// Construct a `StreamId` for a known aggregate instance id.
    pub fn stream_id_for(&self, id: &str) -> StreamId {
        StreamId::new(self.aggregate_type.clone(), id)
    }

    /// Reconstitute an aggregate from a sequence of stored events,
    /// starting from the default state.
    pub fn reconstitute<Seq>(&self, events: &[StoredEvent<Seq>]) -> A {
        self.reconstitute_from(A::default(), events)
    }

    /// Reconstitute an aggregate from a starting state by applying events.
    /// Used with snapshots: start from the snapshot state and replay only
    /// events that occurred after the snapshot.
    pub fn reconstitute_from<Seq>(&self, mut agg: A, events: &[StoredEvent<Seq>]) -> A {
        for stored in events {
            self.apply_event(&mut agg, stored.event.as_ref());
        }
        agg
    }

    /// Apply a single event to an aggregate using the registered applier.
    fn apply_event(&self, agg: &mut A, event: &dyn DomainEvent) {
        let type_id = (*event).type_id();
        if let Some(applier) = self.appliers.get(&type_id) {
            applier(agg, event);
        }
    }
}

pub struct StreamDefinitionBuilder<A> {
    aggregate_type: AggregateType,
    extractors: HashMap<TypeId, StreamIdExtractor>,
    appliers: HashMap<TypeId, EventApplier<A>>,
}

impl<A: Aggregate> StreamDefinitionBuilder<A> {
    /// Register an event type with a stream-id extractor.
    ///
    /// Enforces at compile time that `A: ApplyEvent<E>` — the aggregate
    /// must handle this event type.
    pub fn with<E: DomainEvent>(mut self, extract: fn(&E) -> Option<String>) -> Self
    where
        A: ApplyEvent<E>,
    {
        let type_id = TypeId::of::<E>();

        let extractor: StreamIdExtractor = Arc::new(move |event: &dyn DomainEvent| {
            let any = event as &dyn Any;
            any.downcast_ref::<E>().and_then(extract)
        });
        self.extractors.insert(type_id, extractor);

        let applier: EventApplier<A> = Box::new(move |agg: &mut A, event: &dyn DomainEvent| {
            let any = event as &dyn Any;
            if let Some(typed) = any.downcast_ref::<E>() {
                agg.apply(typed);
            }
        });
        self.appliers.insert(type_id, applier);

        self
    }

    pub fn build(self) -> StreamDefinition<A> {
        StreamDefinition {
            aggregate_type: self.aggregate_type,
            extractors: self.extractors,
            appliers: self.appliers,
        }
    }
}

/// Type-erased interface for the bus/registry to query which streams
/// an event belongs to, without knowing the aggregate type.
pub trait StreamExtract: Send + Sync {
    fn stream_id(&self, event: &dyn DomainEvent) -> Option<StreamId>;
}

impl<A: Aggregate> StreamExtract for StreamDefinition<A> {
    fn stream_id(&self, event: &dyn DomainEvent) -> Option<StreamId> {
        self.stream_id(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aggregate::traits::{Aggregate, ApplyEvent},
        event::event_metadata::EventMetadata,
    };

    // -- Test aggregates and events --

    #[derive(Default)]
    struct Medium {
        id: String,
        title: String,
        task_count: usize,
    }

    impl Aggregate for Medium {
        type Id = String;
        fn aggregate_type() -> &'static str {
            "Medium"
        }
    }

    #[derive(Debug)]
    struct MediumCreated {
        metadata: EventMetadata,
        medium_id: String,
        title: String,
    }

    impl DomainEvent for MediumCreated {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    impl ApplyEvent<MediumCreated> for Medium {
        fn apply(&mut self, event: &MediumCreated) {
            self.id = event.medium_id.clone();
            self.title = event.title.clone();
        }
    }

    #[derive(Debug)]
    struct MediumUpdated {
        metadata: EventMetadata,
        medium_id: String,
        title: String,
    }

    impl DomainEvent for MediumUpdated {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    impl ApplyEvent<MediumUpdated> for Medium {
        fn apply(&mut self, event: &MediumUpdated) {
            self.title = event.title.clone();
        }
    }

    #[derive(Debug)]
    struct TaskCreated {
        metadata: EventMetadata,
        task_id: String,
        medium_id: Option<String>,
    }

    impl DomainEvent for TaskCreated {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    impl ApplyEvent<TaskCreated> for Medium {
        fn apply(&mut self, _event: &TaskCreated) {
            self.task_count += 1;
        }
    }

    // -- Second aggregate for multi-stream tests --

    #[derive(Default)]
    struct Task {
        id: String,
    }

    impl Aggregate for Task {
        type Id = String;
        fn aggregate_type() -> &'static str {
            "Task"
        }
    }

    impl ApplyEvent<TaskCreated> for Task {
        fn apply(&mut self, event: &TaskCreated) {
            self.id = event.task_id.clone();
        }
    }

    // -- Unrelated event --

    #[derive(Debug)]
    struct UnrelatedEvent {
        metadata: EventMetadata,
    }

    impl DomainEvent for UnrelatedEvent {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    fn medium_stream() -> StreamDefinition<Medium> {
        StreamDefinition::<Medium>::builder()
            .with::<MediumCreated>(|e| Some(e.medium_id.clone()))
            .with::<MediumUpdated>(|e| Some(e.medium_id.clone()))
            .with::<TaskCreated>(|e| e.medium_id.clone())
            .build()
    }

    fn task_stream() -> StreamDefinition<Task> {
        StreamDefinition::<Task>::builder()
            .with::<TaskCreated>(|e| Some(e.task_id.clone()))
            .build()
    }

    // -- StreamDefinition tests --

    #[test]
    fn stream_id_extracts_for_registered_event() {
        let stream = medium_stream();
        let event = MediumCreated {
            metadata: EventMetadata::default(),
            medium_id: "abc".to_string(),
            title: "Test".to_string(),
        };
        let id = stream.stream_id(&event).unwrap();
        assert_eq!(id.aggregate_type().name(), "Medium");
        assert_eq!(id.id(), "abc");
    }

    #[test]
    fn stream_id_returns_none_for_unregistered_event() {
        let stream = medium_stream();
        let event = UnrelatedEvent {
            metadata: EventMetadata::default(),
        };
        assert!(stream.stream_id(&event).is_none());
    }

    #[test]
    fn stream_id_returns_none_when_extractor_returns_none() {
        let stream = medium_stream();
        // TaskCreated with no medium_id
        let event = TaskCreated {
            metadata: EventMetadata::default(),
            task_id: "t1".to_string(),
            medium_id: None,
        };
        assert!(stream.stream_id(&event).is_none());
    }

    #[test]
    fn reconstitute_applies_all_events() {
        let stream = medium_stream();
        let events = vec![
            StoredEvent {
                sequence: 1i64,
                event: Box::new(MediumCreated {
                    metadata: EventMetadata::default(),
                    medium_id: "abc".to_string(),
                    title: "Original".to_string(),
                }) as Box<dyn DomainEvent>,
            },
            StoredEvent {
                sequence: 2,
                event: Box::new(MediumUpdated {
                    metadata: EventMetadata::default(),
                    medium_id: "abc".to_string(),
                    title: "Updated".to_string(),
                }),
            },
            StoredEvent {
                sequence: 3,
                event: Box::new(TaskCreated {
                    metadata: EventMetadata::default(),
                    task_id: "t1".to_string(),
                    medium_id: Some("abc".to_string()),
                }),
            },
        ];

        let medium: Medium = stream.reconstitute(&events);
        assert_eq!(medium.id, "abc");
        assert_eq!(medium.title, "Updated");
        assert_eq!(medium.task_count, 1);
    }

    #[test]
    fn stream_id_for_constructs_id() {
        let stream = medium_stream();
        let id = stream.stream_id_for("abc");
        assert_eq!(id.aggregate_type().name(), "Medium");
        assert_eq!(id.id(), "abc");
        assert_eq!(id.to_storage_key(), "Medium-abc");
    }

    // -- AggregateRepository tests --

    use crate::aggregate::{
        event_store::fixtures::MockAggregateEventStore, repository::AggregateRepository,
    };

    #[test]
    fn streams_for_matches_multiple_streams() {
        let mut repo = AggregateRepository::new(Arc::new(MockAggregateEventStore::new()));
        repo.register(medium_stream());
        repo.register(task_stream());

        let event = TaskCreated {
            metadata: EventMetadata::default(),
            task_id: "t1".to_string(),
            medium_id: Some("abc".to_string()),
        };
        let streams = repo.streams_for(&event);
        assert_eq!(streams.len(), 2);

        let categories: Vec<&str> = streams.iter().map(|s| s.aggregate_type().name()).collect();
        assert!(categories.contains(&"Medium"));
        assert!(categories.contains(&"Task"));
    }

    #[test]
    fn streams_for_matches_none() {
        let mut repo = AggregateRepository::new(Arc::new(MockAggregateEventStore::new()));
        repo.register(medium_stream());

        let event = UnrelatedEvent {
            metadata: EventMetadata::default(),
        };
        assert!(repo.streams_for(&event).is_empty());
    }

    #[test]
    fn streams_for_matches_one_only() {
        let mut repo = AggregateRepository::new(Arc::new(MockAggregateEventStore::new()));
        repo.register(medium_stream());
        repo.register(task_stream());

        let event = MediumCreated {
            metadata: EventMetadata::default(),
            medium_id: "abc".to_string(),
            title: "Test".to_string(),
        };
        let streams = repo.streams_for(&event);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].aggregate_type().name(), "Medium");
    }

    #[tokio::test]
    async fn load_reconstitutes_aggregate() {
        let store = Arc::new(MockAggregateEventStore::new());
        store.add_event(
            "Medium-abc",
            1,
            Box::new(|| {
                Box::new(MediumCreated {
                    metadata: EventMetadata::default(),
                    medium_id: "abc".to_string(),
                    title: "Original".to_string(),
                })
            }),
        );
        store.add_event(
            "Medium-abc",
            2,
            Box::new(|| {
                Box::new(MediumUpdated {
                    metadata: EventMetadata::default(),
                    medium_id: "abc".to_string(),
                    title: "Updated".to_string(),
                })
            }),
        );

        let mut repo = AggregateRepository::new(store);
        repo.register(medium_stream());

        let (medium, version) = repo.load::<Medium>(&"abc".to_string()).await.unwrap();
        assert_eq!(medium.id, "abc");
        assert_eq!(medium.title, "Updated");
        assert_eq!(version, 2);
    }

    #[tokio::test]
    async fn load_returns_error_for_unregistered_aggregate() {
        let store = Arc::new(MockAggregateEventStore::new());
        let repo = AggregateRepository::new(store);

        let result = repo.load::<Medium>(&"abc".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn load_returns_default_for_empty_stream() {
        let store = Arc::new(MockAggregateEventStore::new());
        let mut repo = AggregateRepository::new(store);
        repo.register(medium_stream());

        let (medium, version) = repo
            .load::<Medium>(&"nonexistent".to_string())
            .await
            .unwrap();
        assert_eq!(medium.id, "");
        assert_eq!(medium.title, "");
        assert_eq!(version, 0);
    }
}
