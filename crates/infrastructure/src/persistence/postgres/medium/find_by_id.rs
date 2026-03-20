use std::path::PathBuf;

use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use domain::{
    error::DomainResult,
    medium::{
        storage::FileLocation, Dimensions, Filename, GpsCoordinates, Medium, MediumId, MediumItem,
        Priority,
    },
    user::UserId,
};
use futures_util::StreamExt;
use tracing::{error, info};
use uuid::Uuid;

use crate::persistence::postgres::{
    groups::{GroupedRow, GroupedStreamExt},
    medium::{
        types::{MediumItemTypeDb, MediumTypeDb, StorageTierDb},
        PostgresMediumRepository,
    },
    repo_error,
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct FindMediumRow {
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
    pub(super) async fn find_by_id_impl(
        &self,
        id: MediumId,
        user_id: UserId,
    ) -> DomainResult<Option<Medium>> {
        let stream = sqlx::query_as!(
            FindMediumRow,
            r#"
            SELECT
                m.id,
                m.owner_id,
                m.medium_type as "medium_type: MediumTypeDb",
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
                mi.medium_item_type as "medium_item_type: MediumItemTypeDb",
                mi.mime,
                mi.filename,
                mi.size,
                mi.priority,
                mi.width,
                mi.height,
                mi.created_at as item_created_at,
                mi.updated_at as item_updated_at,
                l.variant as "storage_tier: StorageTierDb",
                l.path as relative_path
            FROM media m
            JOIN medium_items mi ON mi.medium_id = m.id AND mi.deleted_at IS NULL
            JOIN locations l ON l.item_id = mi.id
            WHERE m.id = $1 AND m.owner_id = $2
            ORDER BY mi.priority ASC, mi.id, l.variant
            "#,
            id,
            user_id
        )
        .fetch(&self.pool)
        .grouped()
        .into_future()
        .await
        .0
        .transpose()
        .map_err(repo_error)?;

        info!("Found medium by id: {}", id);

        Ok(stream)
    }
}

impl GroupedRow<Medium, Uuid> for FindMediumRow {
    fn key(&self) -> &Uuid {
        &self.id
    }

    fn start_parent(self) -> Medium {
        Medium::from(&self)
    }

    fn push_into(self, parent: &mut Medium) {
        let previous_item = parent.items.last_mut().unwrap();
        if previous_item.id == self.item_id {
            previous_item.locations.push(FileLocation::from(&self));
        } else {
            parent.items.push(MediumItem::from(&self));
        }
    }
}

impl From<&FindMediumRow> for FileLocation {
    fn from(row: &FindMediumRow) -> Self {
        FileLocation {
            storage_tier: row.storage_tier.into(),
            relative_path: PathBuf::from(&row.relative_path),
        }
    }
}

impl From<&FindMediumRow> for MediumItem {
    fn from(row: &FindMediumRow) -> Self {
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

impl From<&FindMediumRow> for Medium {
    fn from(row: &FindMediumRow) -> Self {
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

        Medium {
            id: row.id,
            owner_id: row.owner_id,
            medium_type: row.medium_type.into(),
            leading_item_id: row.leading_item_id,
            taken_at,
            camera_make: row.camera_make.clone(),
            camera_model: row.camera_model.clone(),
            gps_coordinates,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            items: vec![MediumItem::from(row)],
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::{stream, TryStreamExt};

    use super::*;

    fn create_test_row(
        medium_id: Uuid,
        item_id: Uuid,
        storage_tier: StorageTierDb,
        relative_path: &str,
    ) -> FindMediumRow {
        let now = Utc::now().naive_utc();
        FindMediumRow {
            id: medium_id,
            owner_id: Uuid::new_v4(),
            medium_type: MediumTypeDb::Photo,
            leading_item_id: item_id,
            taken_at: None,
            taken_at_timezone: None,
            camera_make: None,
            camera_model: None,
            gps_latitude: None,
            gps_longitude: None,
            gps_altitude: None,
            created_at: now,
            updated_at: now,
            item_id,
            medium_item_type: MediumItemTypeDb::Original,
            mime: "image/jpeg".to_string(),
            filename: "test.jpg".to_string(),
            size: 1024,
            priority: 0,
            width: Some(1920),
            height: Some(1080),
            item_created_at: now,
            item_updated_at: now,
            storage_tier,
            relative_path: relative_path.to_string(),
        }
    }

    #[tokio::test]
    async fn grouped_stream_aggregates_rows_into_medium_with_items_and_locations() {
        // arrange
        let medium1_id = Uuid::new_v4();
        let medium2_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let item3_id = Uuid::new_v4();

        let rows: Vec<Result<FindMediumRow, sqlx::Error>> = vec![
            // Medium 1: two items, first item has two locations
            Ok(create_test_row(
                medium1_id,
                item1_id,
                StorageTierDb::Originals,
                "photos/a.jpg",
            )),
            Ok(create_test_row(
                medium1_id,
                item1_id,
                StorageTierDb::Cache,
                "cache/a.jpg",
            )),
            Ok(create_test_row(
                medium1_id,
                item2_id,
                StorageTierDb::Originals,
                "photos/b.jpg",
            )),
            // Medium 2: one item with one location
            Ok(create_test_row(
                medium2_id,
                item3_id,
                StorageTierDb::Originals,
                "photos/c.jpg",
            )),
        ];

        // act
        let media: Vec<Medium> = stream::iter(rows).grouped().try_collect().await.unwrap();

        // assert
        assert_eq!(media.len(), 2);

        let medium1 = &media[0];
        assert_eq!(medium1.id, medium1_id);
        assert_eq!(medium1.items.len(), 2);
        assert_eq!(medium1.items[0].locations.len(), 2);
        assert_eq!(medium1.items[1].locations.len(), 1);

        let medium2 = &media[1];
        assert_eq!(medium2.id, medium2_id);
        assert_eq!(medium2.items.len(), 1);
        assert_eq!(medium2.items[0].locations.len(), 1);
    }
}
