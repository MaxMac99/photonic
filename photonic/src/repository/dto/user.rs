use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub given_name: Option<String>,
    pub quota: i64,
    pub quota_used: i64,
}
