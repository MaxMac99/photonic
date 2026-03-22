use std::borrow::Cow;

use async_trait::async_trait;
use domain::metadata::events::MetadataEvent;
use sqlx::PgPool;
use tracing::debug;

use super::{Projection, ProjectionResult};

/// Projection that maintains the metadata read model table.
pub struct MetadataProjection {
    pool: PgPool,
}

impl MetadataProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projection<MetadataEvent> for MetadataProjection {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("metadata_read_model")
    }

    async fn handle(
        &self,
        event: &MetadataEvent,
        global_sequence: i64,
    ) -> ProjectionResult<()> {
        debug!(
            global_sequence = global_sequence,
            "MetadataProjection handling event"
        );

        match event {
            MetadataEvent::ExtractionStarted(_) => {
                // No read model update needed
            }
            MetadataEvent::Extracted(_e) => {
                // TODO: UPSERT INTO metadata
            }
            MetadataEvent::ExtractionFailed(_) => {
                // No read model update needed
            }
        }

        Ok(())
    }
}
