use std::{any::Any, collections::HashMap, sync::Arc};

use crate::{
    aggregate::{
        event_store::AggregateEventStore,
        snapshot_store::SnapshotStore,
        traits::{Aggregate, AggregateType},
    },
    error,
    event::domain_event::DomainEvent,
    persistence::sequence::Sequence,
    stream::{
        definition::{StreamDefinition, StreamExtract},
        stream_id::StreamId,
    },
};

/// Registration for a single aggregate type, holding the stream definition
/// and optional snapshot store.
struct AggregateRegistration {
    stream: Arc<dyn Any + Send + Sync>,
    snapshot: Option<Arc<dyn Any + Send + Sync>>,
}

/// Single read-only repository for loading all aggregate types.
///
/// Stream definitions are registered directly on the repository. Each
/// definition provides both the extraction logic (for the bus to determine
/// stream membership on publish) and the reconstitution logic (for loading
/// aggregates from their event streams).
///
/// Optionally supports snapshots: if a snapshot store is provided for an
/// aggregate type, `load` tries the snapshot first and only replays events
/// after the snapshot's version.
///
/// Writing is not the repository's concern — events are published through
/// the event bus, which handles persistence and stream linking automatically.
pub struct AggregateRepository<Seq: Sequence> {
    store: Arc<dyn AggregateEventStore<Seq>>,
    registrations: HashMap<AggregateType, AggregateRegistration>,
    extractors: Vec<Arc<dyn StreamExtract>>,
}

impl<Seq: Sequence> AggregateRepository<Seq> {
    pub fn new(store: Arc<dyn AggregateEventStore<Seq>>) -> Self {
        Self {
            store,
            registrations: HashMap::new(),
            extractors: Vec::new(),
        }
    }

    /// Register a stream definition for an aggregate type without snapshots.
    pub fn register<A: Aggregate>(&mut self, stream: StreamDefinition<A>) {
        let agg_type = AggregateType::of::<A>();
        let arc: Arc<StreamDefinition<A>> = Arc::new(stream);
        self.extractors.push(arc.clone());
        self.registrations.insert(
            agg_type,
            AggregateRegistration {
                stream: arc,
                snapshot: None,
            },
        );
    }

    /// Register a stream definition with a snapshot store for an aggregate type.
    pub fn register_with_snapshots<A: Aggregate>(
        &mut self,
        stream: StreamDefinition<A>,
        snapshot_store: Arc<dyn SnapshotStore<A, Seq>>,
    ) {
        let agg_type = AggregateType::of::<A>();
        let arc: Arc<StreamDefinition<A>> = Arc::new(stream);
        self.extractors.push(arc.clone());
        // Wrap the typed Arc in another Arc so it becomes Arc<Arc<dyn SnapshotStore>>
        // which implements Any for type-erased storage.
        let snapshot_any: Arc<dyn Any + Send + Sync> = Arc::new(snapshot_store);
        self.registrations.insert(
            agg_type,
            AggregateRegistration {
                stream: arc,
                snapshot: Some(snapshot_any),
            },
        );
    }

    /// Load and reconstitute an aggregate from its event stream.
    ///
    /// If a snapshot store is registered, tries to load a snapshot first
    /// and replays only events after the snapshot's version.
    ///
    /// Returns the aggregate and the version (sequence of the last event).
    pub async fn load<A: Aggregate>(&self, id: &A::Id) -> error::Result<(A, Seq)> {
        let reg = self.registration::<A>()?;
        let stream_def = reg
            .stream
            .downcast_ref::<StreamDefinition<A>>()
            .expect("type mismatch in aggregate registration");
        let stream_id = stream_def.stream_id_for(&id.to_string());

        // Try loading a snapshot
        let (initial_state, after_version) = if let Some(snapshot_any) = &reg.snapshot {
            let snapshot_store = snapshot_any
                .downcast_ref::<Arc<dyn SnapshotStore<A, Seq>>>()
                .expect("type mismatch in snapshot store registration");
            match snapshot_store.load(&stream_id).await? {
                Some(snapshot) => {
                    let version = snapshot.version;
                    (snapshot.state, version)
                }
                None => (A::default(), Seq::default()),
            }
        } else {
            (A::default(), Seq::default())
        };

        // Load events after the snapshot version
        let events = self.store.load_stream(&stream_id, after_version).await?;

        let version = events.last().map(|e| e.sequence).unwrap_or(after_version);

        let aggregate = stream_def.reconstitute_from(initial_state, &events);
        Ok((aggregate, version))
    }

    /// Returns the registered stream extractors.
    pub fn extractors(&self) -> &[Arc<dyn StreamExtract>] {
        &self.extractors
    }

    /// Returns all StreamIds an event belongs to across registered definitions.
    pub fn streams_for(&self, event: &dyn DomainEvent) -> Vec<StreamId> {
        self.extractors
            .iter()
            .filter_map(|s| s.stream_id(event))
            .collect()
    }

    fn registration<A: Aggregate>(&self) -> error::Result<&AggregateRegistration> {
        let agg_type = AggregateType::of::<A>();
        self.registrations
            .get(&agg_type)
            .ok_or_else(|| error::EventSourcingError::Store {
                message: format!(
                    "No stream definition registered for aggregate '{}'",
                    A::aggregate_type()
                ),
            })
    }
}
