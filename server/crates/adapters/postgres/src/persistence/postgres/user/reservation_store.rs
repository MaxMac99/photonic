use async_trait::async_trait;
use byte_unit::Byte;
use chrono::{DateTime, Utc};
use kernel::error::DomainResult;
use sqlx::PgPool;
use user::{application::ports::QuotaReservationStore, domain::QuotaReservation};
use uuid::Uuid;

use crate::persistence::postgres::repo_error;

pub struct PostgresQuotaReservationStore {
    pool: PgPool,
}

impl PostgresQuotaReservationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuotaReservationStore for PostgresQuotaReservationStore {
    async fn insert(&self, reservation: &QuotaReservation) -> DomainResult<()> {
        sqlx::query!(
            "INSERT INTO quota_reservations (id, user_id, bytes, created_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5)",
            reservation.id,
            reservation.user_id,
            reservation.bytes.as_u64() as i64,
            reservation.created_at,
            reservation.expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(())
    }

    async fn claim(&self, id: Uuid) -> DomainResult<bool> {
        let result = sqlx::query!("DELETE FROM quota_reservations WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(repo_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_expired(&self, cutoff: DateTime<Utc>) -> DomainResult<Vec<QuotaReservation>> {
        let rows = sqlx::query!(
            "SELECT id, user_id, bytes, created_at, expires_at \
             FROM quota_reservations WHERE expires_at <= $1",
            cutoff,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_error)?;

        Ok(rows
            .into_iter()
            .map(|row| QuotaReservation {
                id: row.id,
                user_id: row.user_id,
                bytes: Byte::from_u64(row.bytes as u64),
                created_at: row.created_at,
                expires_at: row.expires_at,
            })
            .collect())
    }
}
