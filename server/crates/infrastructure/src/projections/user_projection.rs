use async_trait::async_trait;
use domain::user::events::{
    QuotaCommittedEvent, QuotaReleasedEvent, QuotaReservedEvent, UserCreatedEvent, UserUpdatedEvent,
};
use event_sourcing::{
    error::{EventSourcingError, Result},
    projection::handler::ProjectionHandler,
};
use sqlx::{Postgres, Transaction};
use tracing::info;

use super::{register_event, RegisterProjection};

/// Projection that maintains the users read model table.
pub struct UserProjection;

impl UserProjection {
    pub fn new() -> Self {
        Self
    }
}

impl RegisterProjection for UserProjection {
    fn register(
        bus: &super::PgProjectionBus,
        registry: &mut super::EventTypeRegistry,
    ) -> Result<()> {
        register_event::<UserCreatedEvent, _>(bus, registry, Self::new())?;
        register_event::<UserUpdatedEvent, _>(bus, registry, Self::new())?;
        register_event::<QuotaReservedEvent, _>(bus, registry, Self::new())?;
        register_event::<QuotaCommittedEvent, _>(bus, registry, Self::new())?;
        register_event::<QuotaReleasedEvent, _>(bus, registry, Self::new())?;
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<UserCreatedEvent, i64, Transaction<'static, Postgres>> for UserProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &UserCreatedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, email, quota, quota_used, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 0, NOW(), NOW()) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(event.user_id)
        .bind(&event.username)
        .bind(&event.email)
        .bind(event.quota.as_u64() as i64)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert user: {}", e),
        })?;

        info!(user_id = %event.user_id, "UserProjection: user created");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<UserUpdatedEvent, i64, Transaction<'static, Postgres>> for UserProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &UserUpdatedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE users SET \
             username = COALESCE($2, username), \
             email = COALESCE($3, email), \
             quota = COALESCE($4, quota), \
             updated_at = NOW() \
             WHERE id = $1",
        )
        .bind(event.user_id)
        .bind(&event.new_username)
        .bind(&event.new_email)
        .bind(event.new_quota.map(|q| q.as_u64() as i64))
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to update user: {}", e),
        })?;

        info!(user_id = %event.user_id, "UserProjection: user updated");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<QuotaReservedEvent, i64, Transaction<'static, Postgres>> for UserProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &QuotaReservedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query("UPDATE users SET quota_used = $2, updated_at = NOW() WHERE id = $1")
            .bind(event.user_id)
            .bind(event.quota_used_after.as_u64() as i64)
            .execute(&mut **tx)
            .await
            .map_err(|e| EventSourcingError::Projection {
                message: format!("Failed to update quota_used (reserved): {}", e),
            })?;

        info!(user_id = %event.user_id, "UserProjection: quota reserved");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<QuotaCommittedEvent, i64, Transaction<'static, Postgres>>
    for UserProjection
{
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        _event: &QuotaCommittedEvent,
        _sequence: i64,
        _tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        // No-op: quota was already reserved
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<QuotaReleasedEvent, i64, Transaction<'static, Postgres>> for UserProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &QuotaReleasedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query("UPDATE users SET quota_used = $2, updated_at = NOW() WHERE id = $1")
            .bind(event.user_id)
            .bind(event.quota_used_after.as_u64() as i64)
            .execute(&mut **tx)
            .await
            .map_err(|e| EventSourcingError::Projection {
                message: format!("Failed to update quota_used (released): {}", e),
            })?;

        info!(user_id = %event.user_id, "UserProjection: quota released");
        Ok(())
    }
}
