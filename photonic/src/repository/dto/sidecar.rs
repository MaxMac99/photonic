use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{model::StoreLocation, repository::dto::medium::Medium};

#[derive(Debug, Queryable, Associations, Identifiable, Selectable)]
#[diesel(belongs_to(Medium))]
#[diesel(table_name = crate::schema::sidecars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Sidecar {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub mime: String,
    pub filename: String,
    pub path: String,
    pub size: i64,
    pub location: StoreLocation,
    pub priority: i32,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::sidecars)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSidecar {
    pub medium_id: Uuid,
    pub mime: String,
    pub filename: String,
    pub path: String,
    pub size: i64,
    pub location: StoreLocation,
    pub priority: i32,
    pub last_saved: NaiveDateTime,
}
