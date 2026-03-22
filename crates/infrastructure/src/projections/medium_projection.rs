use std::borrow::Cow;

use async_trait::async_trait;
use domain::medium::events::MediumEvent;
use sqlx::PgPool;
use tracing::debug;

use super::{Projection, ProjectionResult};

/// Projection that maintains the media, medium_items, and locations read model tables.
pub struct MediumProjection {
    pool: PgPool,
}

impl MediumProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projection<MediumEvent> for MediumProjection {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("medium_read_model")
    }

    async fn handle(
        &self,
        event: &MediumEvent,
        global_sequence: i64,
    ) -> ProjectionResult<()> {
        debug!(
            global_sequence = global_sequence,
            "MediumProjection handling event"
        );

        match event {
            MediumEvent::MediumCreated(_e) => {
                // TODO: INSERT INTO media, medium_items, locations
            }
            MediumEvent::MediumItemCreated(_e) => {
                // TODO: INSERT INTO medium_items, locations
            }
            MediumEvent::MediumUpdated(_e) => {
                // TODO: UPDATE media SET taken_at, camera_make, camera_model, gps_*
            }
        }

        Ok(())
    }
}
