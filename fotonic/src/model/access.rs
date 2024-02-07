use bson::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Access {
    pub owner: Uuid,
}
