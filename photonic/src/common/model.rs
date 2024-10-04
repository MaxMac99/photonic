use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
pub enum StoreLocation {
    Originals,
    Cache,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum DateDirection {
    #[serde(rename = "DESC")]
    NewestFirst,
    #[serde(rename = "ASC")]
    OldestFirst,
}
