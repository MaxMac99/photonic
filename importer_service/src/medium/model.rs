use chrono::{DateTime, FixedOffset};

use common::medium::MediumType;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct CreateMediumInput {
    pub filename: String,
    pub extension: String,
    #[serde(default = "default_prio")]
    pub priority: i32,
    #[serde(default)]
    pub tags: Vec<String>,
    pub medium_type: Option<MediumType>,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

fn default_prio() -> i32 {
    10
}
