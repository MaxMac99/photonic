use crate::domain::medium::{MediumItemType, MediumType, StorageTier};

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "medium_type_enum", rename_all = "snake_case")]
pub enum MediumTypeDb {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

impl From<MediumType> for MediumTypeDb {
    fn from(mt: MediumType) -> Self {
        match mt {
            MediumType::Photo => MediumTypeDb::Photo,
            MediumType::Video => MediumTypeDb::Video,
            MediumType::LivePhoto => MediumTypeDb::LivePhoto,
            MediumType::Vector => MediumTypeDb::Vector,
            MediumType::Sequence => MediumTypeDb::Sequence,
            MediumType::Gif => MediumTypeDb::Gif,
            MediumType::Other => MediumTypeDb::Other,
        }
    }
}

impl From<MediumTypeDb> for MediumType {
    fn from(db: MediumTypeDb) -> Self {
        match db {
            MediumTypeDb::Photo => MediumType::Photo,
            MediumTypeDb::Video => MediumType::Video,
            MediumTypeDb::LivePhoto => MediumType::LivePhoto,
            MediumTypeDb::Vector => MediumType::Vector,
            MediumTypeDb::Sequence => MediumType::Sequence,
            MediumTypeDb::Gif => MediumType::Gif,
            MediumTypeDb::Other => MediumType::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "medium_item_type_enum", rename_all = "snake_case")]
pub enum MediumItemTypeDb {
    Original,
    Edit,
    Preview,
    Sidecar,
}

impl From<MediumItemType> for MediumItemTypeDb {
    fn from(mit: MediumItemType) -> Self {
        match mit {
            MediumItemType::Original => MediumItemTypeDb::Original,
            MediumItemType::Edit => MediumItemTypeDb::Edit,
            MediumItemType::Preview => MediumItemTypeDb::Preview,
            MediumItemType::Sidecar => MediumItemTypeDb::Sidecar,
        }
    }
}

impl From<MediumItemTypeDb> for MediumItemType {
    fn from(db: MediumItemTypeDb) -> Self {
        match db {
            MediumItemTypeDb::Original => MediumItemType::Original,
            MediumItemTypeDb::Edit => MediumItemType::Edit,
            MediumItemTypeDb::Preview => MediumItemType::Preview,
            MediumItemTypeDb::Sidecar => MediumItemType::Sidecar,
        }
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "store_location_enum", rename_all = "snake_case")]
pub enum StorageTierDb {
    Originals,
    Cache,
    Temp,
}

impl From<StorageTier> for StorageTierDb {
    fn from(tier: StorageTier) -> Self {
        match tier {
            StorageTier::Permanent => StorageTierDb::Originals,
            StorageTier::Cache => StorageTierDb::Cache,
            StorageTier::Temporary => StorageTierDb::Temp,
        }
    }
}

impl From<StorageTierDb> for StorageTier {
    fn from(db: StorageTierDb) -> Self {
        match db {
            StorageTierDb::Originals => StorageTier::Permanent,
            StorageTierDb::Cache => StorageTier::Cache,
            StorageTierDb::Temp => StorageTier::Temporary,
        }
    }
}
