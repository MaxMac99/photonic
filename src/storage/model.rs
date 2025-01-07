use crate::config::StorageConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum::Display;

#[derive(
    Display, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "store_location_enum", rename_all = "lowercase")]
pub enum StorageVariant {
    Originals,
    Cache,
    Temp,
}

impl StorageVariant {
    pub fn speed(&self) -> u8 {
        match self {
            StorageVariant::Originals => 0,
            StorageVariant::Cache => 1,
            StorageVariant::Temp => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, utoipa::ToSchema)]
pub struct StorageLocation {
    pub variant: StorageVariant,
    #[schema(value_type = String)]
    pub path: PathBuf,
}

impl StorageLocation {
    pub fn full_path(&self, config: &StorageConfig) -> PathBuf {
        let path = self.path.clone();
        match self.variant {
            StorageVariant::Originals => config.base_path.join(path),
            StorageVariant::Cache => config.cache_path.join(path),
            StorageVariant::Temp => config.tmp_path.join(path),
        }
    }
}
