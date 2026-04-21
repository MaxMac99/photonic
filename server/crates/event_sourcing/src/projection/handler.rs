use async_trait::async_trait;

use crate::{event::domain_event::DomainEvent, persistence::sequence::Sequence};

/// A projection handler that processes a single event type within a transaction.
///
/// For projections that handle multiple event types, implement this trait
/// once per event type and register each with the bus separately. This
/// ensures full type safety — no downcasting needed.
///
/// Multiple impls on the same struct share the same checkpoint name by default
/// (derived from `type_name::<Self>()`), so they advance as a single cursor
/// through the event stream.
#[async_trait]
pub trait ProjectionHandler<E: DomainEvent, Seq: Sequence, Tx>: Send + Sync + 'static {
    type Error: std::fmt::Display + Send;

    /// Checkpoint key identifying this projection. Defaults to the struct's
    /// type name. All handlers sharing the same name share one checkpoint.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    async fn handle(&self, event: &E, sequence: Seq, tx: &mut Tx) -> Result<(), Self::Error>;
}

/// A catch-all projection handler that receives every event as `&dyn DomainEvent`.
///
/// Used for infrastructure projections that need to process all event types
/// dynamically (e.g. `StreamLinkingProjection`). Business projections should
/// use [`ProjectionHandler`] for type safety.
#[async_trait]
pub trait CatchAllProjectionHandler<Seq: Sequence, Tx>: Send + Sync + 'static {
    type Error: std::fmt::Display + Send;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    async fn handle(
        &self,
        event: &dyn DomainEvent,
        sequence: Seq,
        tx: &mut Tx,
    ) -> Result<(), Self::Error>;
}
