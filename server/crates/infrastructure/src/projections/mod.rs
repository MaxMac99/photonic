mod medium_projection;
mod metadata_projection;
mod task_projection;
mod user_projection;

use async_trait::async_trait;
use domain::event::DomainEvent;
use event_sourcing::{
    bus::EventProcessor, error::EventSourcingError, projection::handler::ProjectionHandler,
};
pub use medium_projection::MediumProjection;
pub use metadata_projection::MetadataProjection;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{Postgres, Transaction};
pub use task_projection::TaskProjection;
pub use user_projection::UserProjection;

use crate::persistence::postgres::events::type_registry::EventTypeRegistry;

pub type PgProjectionBus =
    event_sourcing::bus::projection::ProjectionEventBus<i64, Transaction<'static, Postgres>>;

/// Trait for projections that register all their handlers + event types.
/// Colocates bus registration with event type registry registration so
/// you can't forget one without the other.
pub trait RegisterProjection {
    fn register(
        bus: &PgProjectionBus,
        registry: &mut EventTypeRegistry,
    ) -> event_sourcing::error::Result<()>;
}

/// Register a single projection handler AND its event type in one call.
/// Enforces `Serialize + DeserializeOwned` at compile time — if the event
/// doesn't have serde derives, this won't compile.
pub fn register_event<E, H>(
    bus: &PgProjectionBus,
    registry: &mut EventTypeRegistry,
    handler: H,
) -> event_sourcing::error::Result<()>
where
    E: DomainEvent + Serialize + DeserializeOwned + 'static,
    H: ProjectionHandler<E, i64, Transaction<'static, Postgres>>,
{
    registry.register::<E>();
    bus.register::<E, _>(handler)?;
    Ok(())
}

/// Register a side-effect listener as a checkpointed projection handler.
/// The listener is replayed from its checkpoint during `bus.start()` and
/// receives live events after. The transaction is used only for checkpoint
/// persistence — the listener itself doesn't participate in it.
pub fn register_listener<E, P>(
    bus: &PgProjectionBus,
    registry: &mut EventTypeRegistry,
    processor: P,
) -> event_sourcing::error::Result<()>
where
    E: DomainEvent + Serialize + DeserializeOwned + 'static,
    P: EventProcessor<E>,
{
    registry.register::<E>();
    bus.register::<E, _>(ListenerAdapter(processor))?;
    Ok(())
}

/// Wraps an `EventProcessor` as a `ProjectionHandler` so it can be registered
/// on the projection bus with checkpointing and replay support.
struct ListenerAdapter<P>(P);

#[async_trait]
impl<E, P> ProjectionHandler<E, i64, Transaction<'static, Postgres>> for ListenerAdapter<P>
where
    E: DomainEvent + 'static,
    P: EventProcessor<E>,
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &E,
        _sequence: i64,
        _tx: &mut Transaction<'static, Postgres>,
    ) -> event_sourcing::error::Result<()> {
        self.0
            .process(event)
            .await
            .map_err(|e| EventSourcingError::Bus {
                message: format!("{e}"),
            })
    }
}
