use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::aggregate::event_store::AggregateEventStore;
use crate::aggregate::traits::{Aggregate, AggregateType};
use crate::error;
use crate::event::domain_event::DomainEvent;
use crate::persistence::sequence::Sequence;
use crate::stream::definition::{StreamDefinition, StreamExtract};
use crate::stream::stream_id::StreamId;

/// Single read-only repository for loading all aggregate types.
///
/// Stream definitions are registered directly on the repository. Each
/// definition provides both the extraction logic (for the bus to determine
/// stream membership on publish) and the reconstitution logic (for loading
/// aggregates from their event streams).
///
/// Writing is not the repository's concern — events are published through
/// the event bus, which handles persistence and stream linking automatically.
///
/// # Example
///
/// ```ignore
/// let mut repo = AggregateRepository::new(store);
/// repo.register(
///     StreamDefinition::<Medium>::builder()
///         .with::<MediumCreated>(|e| Some(e.medium_id.to_string()))
///         .build()
/// );
/// let (medium, version) = repo.load::<Medium>(&medium_id).await?;
/// ```
pub struct AggregateRepository<Seq: Sequence> {
    store: Arc<dyn AggregateEventStore<Seq>>,
    streams: HashMap<AggregateType, Arc<dyn Any + Send + Sync>>,
    extractors: Vec<Arc<dyn StreamExtract>>,
}

impl<Seq: Sequence> AggregateRepository<Seq> {
    pub fn new(store: Arc<dyn AggregateEventStore<Seq>>) -> Self {
        Self {
            store,
            streams: HashMap::new(),
            extractors: Vec::new(),
        }
    }

    /// Register a stream definition for an aggregate type.
    pub fn register<A: Aggregate>(&mut self, stream: StreamDefinition<A>) {
        let agg_type = AggregateType::of::<A>();
        let arc: Arc<StreamDefinition<A>> = Arc::new(stream);
        self.extractors.push(arc.clone());
        self.streams.insert(agg_type, arc);
    }

    /// Load and reconstitute an aggregate from its event stream.
    /// Returns the aggregate and the sequence of the last event.
    pub async fn load<A: Aggregate>(&self, id: &A::Id) -> error::Result<(A, Seq)> {
        let stream_def = self.stream_def::<A>()?;
        let stream_id = stream_def.stream_id_for(&id.to_string());
        let events = self.store.load_stream(&stream_id).await?;
        let version = events.last().map(|e| e.sequence).unwrap_or_default();
        let aggregate = stream_def.reconstitute(&events);
        Ok((aggregate, version))
    }

    /// Returns the registered stream extractors. Used to construct a
    /// [`StreamLinkingProjection`](crate::stream::linking_projection::StreamLinkingProjection).
    pub fn extractors(&self) -> &[Arc<dyn StreamExtract>] {
        &self.extractors
    }

    /// Returns all StreamIds an event belongs to across registered definitions.
    /// The bus calls this during publish to determine stream membership.
    pub fn streams_for(&self, event: &dyn DomainEvent) -> Vec<StreamId> {
        self.extractors
            .iter()
            .filter_map(|s| s.stream_id(event))
            .collect()
    }

    fn stream_def<A: Aggregate>(&self) -> error::Result<&StreamDefinition<A>> {
        let agg_type = AggregateType::of::<A>();
        self.streams
            .get(&agg_type)
            .and_then(|arc| arc.downcast_ref::<StreamDefinition<A>>())
            .ok_or_else(|| error::EventSourcingError::Store {
                message: format!(
                    "No stream definition registered for aggregate '{}'",
                    A::aggregate_type()
                ),
            })
    }
}
