use diesel::{Associations, Identifiable, Queryable, Selectable};
use uuid::Uuid;

use crate::model::{Medium, User};

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = owner_id))]
#[diesel(belongs_to(Medium, foreign_key = title_medium))]
#[diesel(table_name = crate::schema::albums)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Album {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub title_medium: Option<Uuid>,
}
