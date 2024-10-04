use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Album {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub title_medium: Option<Uuid>,
}
