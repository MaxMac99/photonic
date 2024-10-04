use crate::config::StorageConfig;
use avro_reference::AvroReferenceSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, AvroReferenceSchema)]
#[avro(referencable)]
pub(super) struct DateTimeTz {
    pub datetime: i64,
    pub timezone: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, AvroReferenceSchema)]
#[sqlx(type_name = "store_location_enum", rename_all = "lowercase")]
pub enum StorageVariant {
    Originals,
    Cache,
    Temp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, AvroReferenceSchema)]
#[avro(referencable)]
pub struct StorageLocation {
    pub variant: StorageVariant,
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
