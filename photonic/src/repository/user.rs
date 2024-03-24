use diesel::{upsert::excluded, ExpressionMethods, RunQueryDsl};

use crate::{
    error::Result,
    repository::{dto::user::User, Repository},
    schema::users,
    service::CreateUserInput,
};

impl Repository {
    pub async fn create_or_update_user(&self, user: CreateUserInput) -> Result<()> {
        let conn = self.pool.get().await?;
        let user = User {
            id: user.id,
            email: user.email,
            username: user.username,
            given_name: user.given_name,
            quota: user.quota as i64,
            quota_used: 0,
        };
        conn.interact(move |conn| {
            diesel::insert_into(users::table)
                .values(user)
                .on_conflict(users::id)
                .do_update()
                .set((
                    users::email.eq(excluded(users::email)),
                    users::username.eq(excluded(users::username)),
                    users::given_name.eq(excluded(users::given_name)),
                    users::quota.eq(excluded(users::quota)),
                ))
                .execute(conn)
        })
        .await??;
        Ok(())
    }
}
