use std::path::PathBuf;

use derive_new::new;
use mime::Mime;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::shared::crypto::Sha256;

#[derive(new, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileLocation {
    pub storage_tier: StorageTier,
    pub relative_path: PathBuf,
}

impl FileLocation {
    pub fn temporary(relative_path: PathBuf) -> Self {
        Self::new(StorageTier::Temporary, relative_path)
    }

    pub fn permanent(relative_path: PathBuf) -> Self {
        Self::new(StorageTier::Permanent, relative_path)
    }

    pub fn cache(relative_path: PathBuf) -> Self {
        Self::new(StorageTier::Cache, relative_path)
    }
}

#[derive(Display, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageTier {
    /// Long term storage, should be backed up
    Permanent,
    /// Short term storage, contains reproducible data
    Temporary,
    /// Cache storage, can be deleted at any time
    Cache,
}

impl StorageTier {
    pub fn speed(&self) -> u8 {
        match self {
            StorageTier::Permanent => 0,
            StorageTier::Cache => 1,
            StorageTier::Temporary => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size_bytes: u64,
    pub mime_type: Mime,
    pub checksum: Sha256,
}
