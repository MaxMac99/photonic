use async_trait::async_trait;
use event_sourcing::{
    error::{EventSourcingError, Result},
    stream::{link_store::StreamLinkStore, stream_id::StreamId},
};
use sqlx::{Postgres, Transaction};

pub struct PostgresStreamLinkStore;

impl PostgresStreamLinkStore {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StreamLinkStore<i64, Transaction<'static, Postgres>> for PostgresStreamLinkStore {
    async fn link(
        &self,
        sequence: i64,
        stream: &StreamId,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO event_streams (global_sequence, stream_category, stream_id, stream_version)
             VALUES ($1, $2, $3, COALESCE(
                 (SELECT MAX(stream_version) + 1
                  FROM event_streams
                  WHERE stream_category = $2 AND stream_id = $3),
                 1
             ))",
        )
        .bind(sequence)
        .bind(stream.aggregate_type().name())
        .bind(stream.id())
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Store {
            message: format!("Failed to link event to stream: {e}"),
        })?;

        Ok(())
    }
}
