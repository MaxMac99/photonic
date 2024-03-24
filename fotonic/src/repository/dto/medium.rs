use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, QueryableByName, Selectable};
use uuid::Uuid;

use crate::{
    model::MediumType,
    repository::dto::{album::Album, user::User},
};

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Queryable, Associations, Selectable, Identifiable)]
#[diesel(belongs_to(Medium))]
#[diesel(belongs_to(Tag))]
#[diesel(table_name = crate::schema::media_tags)]
#[diesel(primary_key(medium_id, tag_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MediumTag {
    pub medium_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Queryable, QueryableByName, Selectable, Associations, Identifiable)]
#[diesel(belongs_to(User, foreign_key = owner_id))]
#[diesel(belongs_to(Album))]
#[diesel(table_name = crate::schema::media)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Medium {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub album_id: Option<Uuid>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::media)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMedium {
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub album_id: Option<Uuid>,
}
