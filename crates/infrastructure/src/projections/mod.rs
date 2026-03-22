pub mod engine;
mod medium_projection;
mod metadata_projection;
mod task_projection;
mod user_projection;

pub use engine::ProjectionEngine;
pub use medium_projection::MediumProjection;
pub use metadata_projection::MetadataProjection;
pub use task_projection::TaskProjection;
pub use user_projection::UserProjection;

use std::borrow::Cow;

use async_trait::async_trait;

use crate::events::StorableEvent;

pub type ProjectionResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// A typed projection that consumes deserialized events and updates read models.
#[async_trait]
pub trait Projection<E: StorableEvent>: Send + Sync {
    /// Unique name for checkpoint tracking
    fn name(&self) -> Cow<'static, str>;

    /// Process a single deserialized event
    async fn handle(&self, event: &E, global_sequence: i64) -> ProjectionResult<()>;
}
