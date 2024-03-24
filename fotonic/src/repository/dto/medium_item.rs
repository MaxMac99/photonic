use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{
    model::{MediumItemType, StoreLocation},
    repository::dto::medium::Medium,
};

#[derive(Debug, Queryable, Selectable, Associations, Identifiable)]
#[diesel(belongs_to(Medium))]
#[diesel(table_name = crate::schema::medium_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MediumItem {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: String,
    pub filename: String,
    pub path: String,
    pub size: i64,
    pub location: StoreLocation,
    pub priority: i32,
    pub timezone: i32,
    pub taken_at: NaiveDateTime,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::medium_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMediumItem {
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: String,
    pub filename: String,
    pub path: String,
    pub size: i64,
    pub location: StoreLocation,
    pub priority: i32,
    pub timezone: i32,
    pub taken_at: NaiveDateTime,
    pub last_saved: NaiveDateTime,
    pub width: i32,
    pub height: i32,
}
