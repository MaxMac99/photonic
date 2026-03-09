use std::path::PathBuf;

use chrono::{DateTime, Utc};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    application::medium::ports::ExpiredTempLocation,
    domain::{
        error::DomainResult,
        medium::storage::FileLocation,
    },
    infrastructure::persistence::postgres::medium::PostgresMediumRepository,
};

#[derive(Debug, sqlx::FromRow)]
struct ExpiredTempLocationRow {
    pub item_id: Uuid,
    pub medium_id: Uuid,
    pub owner_id: Uuid,
    pub temp_path: String,
}

impl PostgresMediumRepository {
    pub(super) async fn find_expired_temp_locations_impl(
        &self,
        created_before: DateTime<Utc>,
    ) -> DomainResult<Vec<ExpiredTempLocation>> {
        debug!("Finding expired temp locations");

        let cutoff = created_before.naive_utc();

        let rows = sqlx::query_as!(
            ExpiredTempLocationRow,
            r#"
            SELECT
                mi.id as item_id,
                mi.medium_id,
                m.owner_id,
                l_temp.path as temp_path
            FROM locations l_temp
            JOIN locations l_perm ON l_temp.item_id = l_perm.item_id
            JOIN medium_items mi ON l_temp.item_id = mi.id
            JOIN media m ON mi.medium_id = m.id
            WHERE l_temp.variant = 'temp'
              AND l_perm.variant = 'originals'
              AND mi.created_at < $1
              AND mi.deleted_at IS NULL
            "#,
            cutoff,
        )
        .fetch_all(&self.pool)
        .await?;

        info!(
            count = rows.len(),
            "Found expired temp locations for cleanup"
        );

        Ok(rows
            .into_iter()
            .map(|row| ExpiredTempLocation {
                medium_id: row.medium_id,
                item_id: row.item_id,
                owner_id: row.owner_id,
                temp_location: FileLocation::temporary(PathBuf::from(row.temp_path)),
            })
            .collect())
    }
}