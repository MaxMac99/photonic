use std::path::PathBuf;

use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use futures_util::TryStreamExt;
use sqlx::QueryBuilder;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    domain::{
        error::DomainResult,
        medium::{
            storage::FileLocation, Dimensions, Filename, GpsCoordinates, MediumFilter, MediumItem,
            MediumListItem, Priority,
        },
    },
    infrastructure::persistence::postgres::{
        groups::{GroupedRow, GroupedStreamExt},
        medium::{
            types::{MediumItemTypeDb, MediumTypeDb, StorageTierDb},
            PostgresMediumRepository,
        },
    },
    shared::SortDirection,
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct FindAllMediumRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumTypeDb,
    pub leading_item_id: Uuid,
    pub taken_at: Option<DateTime<Utc>>,
    pub taken_at_timezone: Option<i32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub item_id: Uuid,
    pub medium_item_type: MediumItemTypeDb,
    pub mime: String,
    pub filename: String,
    pub size: i64,
    pub priority: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub item_created_at: NaiveDateTime,
    pub item_updated_at: NaiveDateTime,
    pub storage_tier: StorageTierDb,
    pub relative_path: String,
}

impl PostgresMediumRepository {
    pub(super) async fn find_all_impl(
        &self,
        filter: MediumFilter,
        user_id: Uuid,
    ) -> DomainResult<Vec<MediumListItem>> {
        debug!("Querying all media with filters");

        let direction_sql = match filter.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };

        let comparison_op = match filter.direction {
            SortDirection::Ascending => ">",
            SortDirection::Descending => "<",
        };

        // Build query with dynamic filters
        let mut query = QueryBuilder::new(
            r#"SELECT DISTINCT
                m.id,
                m.owner_id,
                m.medium_type,
                m.leading_item_id,
                m.taken_at,
                m.taken_at_timezone,
                m.camera_make,
                m.camera_model,
                m.gps_latitude,
                m.gps_longitude,
                m.gps_altitude,
                m.created_at,
                m.updated_at,
                mi.id as item_id,
                mi.medium_item_type,
                mi.mime,
                mi.filename,
                mi.size,
                mi.priority,
                mi.width,
                mi.height,
                mi.created_at as item_created_at,
                mi.updated_at as item_updated_at,
                l.variant as storage_tier,
                l.path as relative_path
            FROM media m
            JOIN medium_items mi ON m.id = mi.medium_id AND mi.deleted_at IS NULL
            JOIN locations l ON mi.id = l.item_id"#,
        );

        // WHERE clauses
        query.push(" WHERE m.owner_id = ");
        query.push_bind(user_id);
        query.push(" AND m.deleted_at IS NULL ");

        // Date range filters
        if let Some(start_date) = filter.start_date {
            query.push(" AND m.taken_at >= ");
            query.push_bind(start_date);
        }
        if let Some(end_date) = filter.end_date {
            query.push(" AND m.taken_at <= ");
            query.push_bind(end_date);
        }

        // Keyset pagination cursor
        if let Some(cursor) = filter.cursor {
            query.push(" AND (m.taken_at, m.id) ");
            query.push(comparison_op);
            query.push(" (");
            query.push_bind(cursor.last_date);
            query.push(", ");
            query.push_bind(cursor.last_id);
            query.push(") ");
        }

        // ORDER BY for keyset pagination
        query.push(" ORDER BY m.taken_at ");
        query.push(direction_sql);
        query.push(", mi.priority DESC, m.id, mi.id, l.variant");

        // LIMIT
        query.push(" LIMIT ");
        query.push_bind(filter.per_page as i64);

        // Execute query
        let media = query
            .build_query_as::<FindAllMediumRow>()
            .fetch(&self.pool)
            .grouped()
            .try_collect::<Vec<MediumListItem>>()
            .await
            .map_err(|err| {
                error!("Failed to load media: {}", err);
                err
            })?;

        info!(count = media.len(), "Media query completed");

        Ok(media)
    }
}

impl GroupedRow<MediumListItem, Uuid> for FindAllMediumRow {
    fn key(&self) -> &Uuid {
        &self.id
    }

    fn start_parent(self) -> MediumListItem {
        MediumListItem::from(&self)
    }

    fn push_into(self, parent: &mut MediumListItem) {
        let previous_item = parent.items.last_mut().unwrap();
        if previous_item.id == self.item_id {
            previous_item.locations.push(FileLocation::from(&self));
        } else {
            parent.items.push(MediumItem::from(&self));
        }
    }
}

impl From<&FindAllMediumRow> for FileLocation {
    fn from(row: &FindAllMediumRow) -> Self {
        FileLocation {
            storage_tier: row.storage_tier.into(),
            relative_path: PathBuf::from(&row.relative_path),
        }
    }
}

impl From<&FindAllMediumRow> for MediumItem {
    fn from(row: &FindAllMediumRow) -> Self {
        let dimensions = match (row.width, row.height) {
            (Some(w), Some(h)) if w > 0 && h > 0 => Dimensions::new(w as u32, h as u32).ok(),
            _ => None,
        };

        MediumItem {
            id: row.item_id,
            medium_id: row.id,
            medium_item_type: row.medium_item_type.into(),
            mime: row.mime.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM),
            filename: Filename::new(&row.filename).expect("Invalid filename in DB"),
            filesize: Byte::from(row.size as u64),
            priority: Priority::new(row.priority),
            dimensions,
            locations: vec![FileLocation::from(row)],
            created_at: row.item_created_at.and_utc(),
            updated_at: row.item_updated_at.and_utc(),
        }
    }
}

impl From<&FindAllMediumRow> for MediumListItem {
    fn from(row: &FindAllMediumRow) -> Self {
        let taken_at = row.taken_at.and_then(|t| {
            row.taken_at_timezone
                .and_then(|tz| FixedOffset::east_opt(tz).map(|offset| t.with_timezone(&offset)))
        });
        let gps_coordinates = Option::zip(row.gps_latitude, row.gps_longitude)
            .map(|(lat, lon)| {
                let coordinates = GpsCoordinates::new(lat, lon, row.gps_altitude);
                if let Err(e) = &coordinates {
                    error!(
                        "Invalid GPS coordinates for medium_id={}: lat={}, lon={}, alt={:?} - {}",
                        row.id, lat, lon, row.gps_altitude, e
                    );
                }
                coordinates.ok()
            })
            .flatten();

        MediumListItem {
            id: row.id,
            owner_id: row.owner_id,
            medium_type: row.medium_type.into(),
            leading_item_id: row.leading_item_id,
            taken_at,
            camera_make: row.camera_make.clone(),
            camera_model: row.camera_model.clone(),
            gps_coordinates,
            created_at: row.item_created_at.and_utc(),
            updated_at: row.item_updated_at.and_utc(),
            items: vec![MediumItem::from(row)],
        }
    }
}
