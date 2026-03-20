use domain::{
    error::DomainResult,
    metadata::{Metadata, MetadataId},
};

use super::{entity::MetadataDb, PostgresMetadataRepository};
use crate::persistence::postgres::repo_error;

impl PostgresMetadataRepository {
    pub(super) async fn find_by_id_impl(&self, id: MetadataId) -> DomainResult<Option<Metadata>> {
        let result = sqlx::query_as::<_, MetadataDb>(
            r#"
            SELECT
                id, medium_id, extracted_at,
                mime_type, file_size, file_modified_at,
                camera_make, camera_model, capture_date, modified_date,
                lens_make, lens_model, exposure_time, f_number, iso, focal_length, flash,
                latitude, longitude, altitude, direction, horizontal_position_error,
                width, height, orientation,
                additional
            FROM metadata
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(result.map(Into::into))
    }
}
