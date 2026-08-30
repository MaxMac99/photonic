use std::sync::Arc;

use byte_unit::Byte;
use chrono::Utc;
use derive_new::new;
use kernel::{
    app_error::{ApplicationError, ApplicationResult},
    error::{ConcurrentModificationSnafu, DomainError, DomainResult, EntityNotFoundSnafu},
    UserId,
};
use snafu::OptionExt;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    application::ports::{PublishUserEvent, QuotaReservationStore, UserRepository},
    domain::{QuotaCommittedEvent, QuotaReleasedEvent, QuotaReservation, QuotaReservedEvent, User},
};

const RETRIES: u32 = 5;

/// Quota operations on the User aggregate produce one of these events; the
/// retry helper returns them so the caller can publish after a successful
/// update.
enum QuotaUserEvent {
    Reserved(QuotaReservedEvent),
    Released(QuotaReleasedEvent),
}

/// Owns quota reservation semantics (ADR 0003/0005): reserve/commit/release
/// plus the expiry sweep. Reservations are short-lived soft locks tracked in
/// a [`QuotaReservationStore`]; a crash never strands quota permanently
/// because the sweep reclaims expired reservations.
#[derive(new)]
pub struct QuotaManager {
    user_repository: Arc<dyn UserRepository>,
    event_bus: Arc<dyn PublishUserEvent>,
    reservation_store: Arc<dyn QuotaReservationStore>,
    reservation_ttl_seconds: u64,
}

impl QuotaManager {
    /// Reserves quota for a user. The reservation record is persisted
    /// before the aggregate is touched, so a crash mid-reserve always
    /// leaves a sweepable record.
    #[instrument(skip(self), fields(user_id = %user_id, bytes = %bytes.as_u64()))]
    pub async fn reserve(
        &self,
        user_id: UserId,
        bytes: Byte,
    ) -> ApplicationResult<QuotaReservation> {
        debug!("Reserving quota");

        let reservation = QuotaReservation::new(user_id, bytes, self.reservation_ttl_seconds);
        self.reservation_store
            .insert(&reservation)
            .await
            .map_err(|e| ApplicationError::Domain { source: e })?;

        match self
            .update_user_with_retry(user_id, |mut user| {
                let event = user.reserve_quota(bytes)?;
                Ok((user, QuotaUserEvent::Reserved(event)))
            })
            .await
        {
            Ok(()) => {
                info!(
                    bytes_reserved = %bytes.as_u64(),
                    reservation_id = %reservation.id,
                    "Quota reserved successfully"
                );
                Ok(reservation)
            }
            Err(e) => {
                if let Err(cleanup_err) = self.reservation_store.claim(reservation.id).await {
                    error!(
                        reservation_id = %reservation.id,
                        error = %cleanup_err,
                        "Failed to remove reservation record after failed reserve"
                    );
                }
                Err(e)
            }
        }
    }

    /// Finalizes a reservation: the reserved bytes become permanently used.
    #[instrument(skip(self, reservation), fields(user_id = %reservation.user_id, bytes = %reservation.bytes.as_u64(), reservation_id = %reservation.id))]
    pub async fn commit(&self, reservation: QuotaReservation) -> ApplicationResult<()> {
        debug!("Committing quota reservation");

        // Claim first so the sweep can never release committed bytes.
        let claimed = self
            .reservation_store
            .claim(reservation.id)
            .await
            .map_err(|e| ApplicationError::Domain { source: e })?;
        if !claimed {
            warn!(
                reservation_id = %reservation.id,
                "Reservation already claimed (committed or swept); skipping commit"
            );
            return Ok(());
        }

        let event =
            QuotaCommittedEvent::new(reservation.user_id, reservation.bytes, reservation.id);
        if let Err(e) = self.event_bus.publish(event).await {
            warn!(
                user_id = %reservation.user_id,
                error = %e,
                "Failed to publish quota committed event"
            );
        }

        info!(
            bytes = %reservation.bytes.as_u64(),
            "Quota committed successfully"
        );
        Ok(())
    }

    /// Releases a reservation without consuming the bytes. Safe to call
    /// after the sweep already reclaimed it (claim-first, idempotent).
    #[instrument(skip(self, reservation), fields(user_id = %reservation.user_id, bytes = %reservation.bytes.as_u64(), reservation_id = %reservation.id))]
    pub async fn release(&self, reservation: QuotaReservation) -> ApplicationResult<()> {
        debug!("Releasing quota reservation");

        let claimed = self
            .reservation_store
            .claim(reservation.id)
            .await
            .map_err(|e| ApplicationError::Domain { source: e })?;
        if !claimed {
            info!(
                reservation_id = %reservation.id,
                "Reservation already claimed (committed or swept); skipping release"
            );
            return Ok(());
        }

        self.update_user_with_retry(reservation.user_id, |mut user| {
            let event = user.release_quota(reservation.bytes, reservation.id);
            Ok((user, QuotaUserEvent::Released(event)))
        })
        .await
    }

    /// Sweep entry point (ADR 0003): releases every reservation whose
    /// expiry has passed. Returns the number of reservations reclaimed.
    #[instrument(skip(self))]
    pub async fn release_expired(&self) -> ApplicationResult<usize> {
        let expired = self
            .reservation_store
            .find_expired(Utc::now())
            .await
            .map_err(|e| ApplicationError::Domain { source: e })?;

        let mut reclaimed = 0;
        for reservation in expired {
            match self.release(reservation).await {
                Ok(()) => reclaimed += 1,
                Err(e) => {
                    error!(
                        error = %e,
                        "CRITICAL: Failed to release expired quota reservation. \
                         Manual intervention may be required"
                    );
                }
            }
        }

        Ok(reclaimed)
    }

    /// Loads the user, applies the mutation, and persists with optimistic
    /// concurrency + bounded backoff retry. Events are published only after
    /// a successful update.
    async fn update_user_with_retry<F>(&self, user_id: UserId, mutate: F) -> ApplicationResult<()>
    where
        F: Fn(User) -> DomainResult<(User, QuotaUserEvent)>,
    {
        let mut last_version = 0;

        for attempt in 0..RETRIES {
            let user = self
                .user_repository
                .find_by_id(user_id)
                .await
                .map_err(|e| ApplicationError::Domain { source: e })?
                .context(EntityNotFoundSnafu {
                    entity: "User",
                    id: user_id,
                })
                .map_err(|e| ApplicationError::Domain { source: e })?;
            last_version = user.version;

            let (user, event) = mutate(user).map_err(|e| ApplicationError::Domain { source: e })?;

            match self.user_repository.update(&user).await {
                Ok(()) => {
                    self.publish(event, user_id).await;
                    return Ok(());
                }
                Err(DomainError::ConcurrentModification { .. }) => {
                    if attempt < RETRIES - 1 {
                        let backoff_ms = 10 * 2_u64.pow(attempt);
                        warn!(
                            user_id = %user_id,
                            attempt = attempt + 1,
                            max_retries = RETRIES,
                            backoff_ms = backoff_ms,
                            version = last_version,
                            "Concurrent modification detected, retrying with backoff"
                        );
                        sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    } else {
                        error!(
                            user_id = %user_id,
                            attempts = RETRIES,
                            version = last_version,
                            "Concurrent modification retry limit exceeded"
                        );
                        return Err(ApplicationError::Domain {
                            source: ConcurrentModificationSnafu {
                                aggregate_id: user_id,
                                expected_version: last_version,
                            }
                            .build(),
                        });
                    }
                }
                Err(e) => return Err(ApplicationError::Domain { source: e }),
            }
        }

        Err(ApplicationError::Domain {
            source: ConcurrentModificationSnafu {
                aggregate_id: user_id,
                expected_version: last_version,
            }
            .build(),
        })
    }

    async fn publish(&self, event: QuotaUserEvent, user_id: UserId) {
        let result = match event {
            QuotaUserEvent::Reserved(event) => self.event_bus.publish(event).await,
            QuotaUserEvent::Released(event) => self.event_bus.publish(event).await,
        };
        if let Err(e) = result {
            warn!(user_id = %user_id, error = %e, "Failed to publish quota event");
        }
    }
}
