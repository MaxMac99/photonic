use crate::{error::Result, state::Transaction};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct MediumItemInfoDb {
    pub id: Uuid,
    pub taken_at: Option<DateTime<Utc>>,
    pub taken_at_timezone: Option<i32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

pub async fn create_medium_item_info(
    transaction: &mut Transaction,
    medium_item_info: MediumItemInfoDb,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO medium_item_info (id, taken_at, taken_at_timezone, camera_make, camera_model) VALUES ($1, $2, $3, $4, $5)",
        medium_item_info.id,
        medium_item_info.taken_at,
        medium_item_info.taken_at_timezone,
        medium_item_info.camera_make,
        medium_item_info.camera_model,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn find_medium_item_info(
    transaction: &mut Transaction,
    id: Uuid,
) -> Result<Option<MediumItemInfoDb>> {
    let medium_item_info = sqlx::query_as!(
        MediumItemInfoDb,
        "SELECT * FROM medium_item_info WHERE id = $1",
        id
    )
    .fetch_optional(&mut **transaction)
    .await?;
    Ok(medium_item_info)
}

pub async fn update_medium_item_info(
    transaction: &mut Transaction,
    medium_item_info: MediumItemInfoDb,
) -> Result<()> {
    sqlx::query!(
        "UPDATE medium_item_info SET \
            taken_at = $1, \
            taken_at_timezone = $2, \
            camera_make = $3, \
            camera_model = $4, \
            width = $5, \
            height = $6 \
        WHERE id = $7",
        medium_item_info.taken_at,
        medium_item_info.taken_at_timezone,
        medium_item_info.camera_make,
        medium_item_info.camera_model,
        medium_item_info.width,
        medium_item_info.height,
        medium_item_info.id,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
