use std::borrow::Cow;

use async_trait::async_trait;
use domain::user::events::UserEvent;
use sqlx::PgPool;
use tracing::debug;

use super::{Projection, ProjectionResult};

/// Projection that maintains the users read model table.
pub struct UserProjection {
    pool: PgPool,
}

impl UserProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projection<UserEvent> for UserProjection {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("user_read_model")
    }

    async fn handle(
        &self,
        event: &UserEvent,
        global_sequence: i64,
    ) -> ProjectionResult<()> {
        debug!(
            global_sequence = global_sequence,
            "UserProjection handling event"
        );

        match event {
            UserEvent::UserCreated(_e) => {
                // TODO: INSERT INTO users
            }
            UserEvent::UserUpdated(_e) => {
                // TODO: UPDATE users SET username, email, quota
            }
            UserEvent::QuotaReserved(_e) => {
                // TODO: UPDATE users SET quota_used
            }
            UserEvent::QuotaCommitted(_e) => {
                // No-op: quota already reserved
            }
            UserEvent::QuotaReleased(_e) => {
                // TODO: UPDATE users SET quota_used (decrease)
            }
        }

        Ok(())
    }
}
