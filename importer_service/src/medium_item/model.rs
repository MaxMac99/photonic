use crate::medium::model::CreateMediumInput;
use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use common::{medium_item::MediumItemType, stream::events::StorageLocation};
use mime_serde_shim::Wrapper as Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct MediumItem {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: Mime,
    pub filename: String,
    pub location: StorageLocation,
    pub filesize: Byte,
    pub priority: i32,
    pub taken_at: Option<DateTime<FixedOffset>>,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMediumItemInput {
    pub filename: String,
    pub extension: String,
    #[serde(default = "default_prio")]
    pub priority: i32,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

fn default_prio() -> i32 {
    10
}

impl From<CreateMediumInput> for CreateMediumItemInput {
    fn from(value: CreateMediumInput) -> Self {
        Self {
            filename: value.filename,
            extension: value.extension,
            priority: value.priority,
            date_taken: value.date_taken,
        }
    }
}
