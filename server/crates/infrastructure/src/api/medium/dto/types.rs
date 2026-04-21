use domain::medium::{MediumItemType, MediumType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediumTypeDto {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

impl From<MediumType> for MediumTypeDto {
    fn from(mt: MediumType) -> Self {
        match mt {
            MediumType::Photo => MediumTypeDto::Photo,
            MediumType::Video => MediumTypeDto::Video,
            MediumType::LivePhoto => MediumTypeDto::LivePhoto,
            MediumType::Vector => MediumTypeDto::Vector,
            MediumType::Sequence => MediumTypeDto::Sequence,
            MediumType::Gif => MediumTypeDto::Gif,
            MediumType::Other => MediumTypeDto::Other,
        }
    }
}

impl From<MediumTypeDto> for MediumType {
    fn from(dto: MediumTypeDto) -> Self {
        match dto {
            MediumTypeDto::Photo => MediumType::Photo,
            MediumTypeDto::Video => MediumType::Video,
            MediumTypeDto::LivePhoto => MediumType::LivePhoto,
            MediumTypeDto::Vector => MediumType::Vector,
            MediumTypeDto::Sequence => MediumType::Sequence,
            MediumTypeDto::Gif => MediumType::Gif,
            MediumTypeDto::Other => MediumType::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MediumItemTypeDto {
    Original,
    Edit,
    Preview,
    Sidecar,
}

impl From<MediumItemType> for MediumItemTypeDto {
    fn from(mit: MediumItemType) -> Self {
        match mit {
            MediumItemType::Original => MediumItemTypeDto::Original,
            MediumItemType::Edit => MediumItemTypeDto::Edit,
            MediumItemType::Preview => MediumItemTypeDto::Preview,
            MediumItemType::Sidecar => MediumItemTypeDto::Sidecar,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StorageTierDto {
    Permanent,
    Temporary,
    Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileLocationDto {
    pub storage_tier: StorageTierDto,
    #[schema(value_type = String)]
    pub relative_path: String,
}
