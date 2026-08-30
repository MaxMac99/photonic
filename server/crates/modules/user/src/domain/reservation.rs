use byte_unit::Byte;
use chrono::{DateTime, Duration, Utc};
use kernel::UserId;
use uuid::Uuid;

/// A short-lived soft lock on quota bytes (ADR 0003).
///
/// Reservations exist so that a crashed process cannot strand reserved
/// bytes forever: every reservation carries an expiry and the sweep
/// releases the ones whose owner never committed or released them.
#[derive(Debug, Clone)]
pub struct QuotaReservation {
    pub id: Uuid,
    pub user_id: UserId,
    pub bytes: Byte,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl QuotaReservation {
    pub fn new(user_id: UserId, bytes: Byte, ttl_seconds: u64) -> Self {
        let created_at = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            bytes,
            created_at,
            expires_at: created_at + Duration::seconds(ttl_seconds as i64),
        }
    }
}
