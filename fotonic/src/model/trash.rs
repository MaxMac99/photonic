use bson::oid::ObjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::model::{FileItem, Medium, MediumItem};

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TrashItem {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub medium_id: ObjectId,
    #[serde_as(as = "bson::DateTime")]
    pub deleted: chrono::DateTime<Utc>,
    pub medium: Option<Medium>,
    pub original: Option<MediumItem>,
    pub edit: Option<MediumItem>,
    pub sidecar: Option<FileItem>,
    pub preview: Option<MediumItem>,
}
