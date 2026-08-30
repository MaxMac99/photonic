use async_trait::async_trait;
use kernel::{error::DomainResult, MediumId, UserId};
use metadata::{application::queries::MetadataQueryPort, domain::Metadata};
use tracing::debug;

use super::{entity::MetadataDb, PostgresMetadataRepository};
use crate::persistence::postgres::repo_error;

/// Read-side port (ADR 0002), ownership-scoped per ADR 0008: the owner
/// check joins the medium projection so users can never read another
/// user's metadata.
#[async_trait]
impl MetadataQueryPort for PostgresMetadataRepository {
    async fn find_by_medium_id(
        &self,
        medium_id: MediumId,
        user_id: UserId,
    ) -> DomainResult<Option<Metadata>> {
        debug!(medium_id = %medium_id, user_id = %user_id, "Querying metadata by medium id");

        let result = sqlx::query_as::<_, MetadataDb>(
            r#"
            SELECT
                m.id, m.medium_id, m.extracted_at,
                m.mime_type, m.file_size, m.file_modified_at,
                m.camera_make, m.camera_model, m.capture_date, m.modified_date,
                m.lens_make, m.lens_model, m.exposure_time, m.f_number, m.iso, m.focal_length, m.flash,
                m.latitude, m.longitude, m.altitude, m.direction, m.horizontal_position_error,
                m.width, m.height, m.orientation,
                m.additional
            FROM metadata m
            JOIN media me ON me.id = m.medium_id
            WHERE m.medium_id = $1 AND me.owner_id = $2 AND me.deleted_at IS NULL
            "#,
        )
        .bind(medium_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(result.map(Into::into))
    }
}
