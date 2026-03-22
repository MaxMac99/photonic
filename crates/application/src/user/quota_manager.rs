use std::{future::Future, sync::Arc, time::Duration};

use byte_unit::Byte;
use derive_new::new;
use domain::{
    error::{ConcurrentModificationSnafu, DomainError, EntityNotFoundSnafu},
    user::{events::UserEvent, QuotaReleasedEvent, QuotaReservedEvent, User, UserId},
};
use snafu::OptionExt;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    error::{ApplicationError, ApplicationResult},
    user::ports::{PublishUserEvent, UserRepository},
};

#[derive(new)]
pub struct QuotaManager {
    user_repository: Arc<dyn UserRepository>,
    event_bus: Arc<dyn PublishUserEvent>,
}

impl QuotaManager {
    #[instrument(skip(self, operation), fields(user_id = %user_id, bytes = %bytes.as_u64()))]
    pub async fn with_quota<F, Fut, T, E>(
        &self,
        user_id: UserId,
        bytes: Byte,
        operation: F,
    ) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: From<ApplicationError>,
    {
        debug!("Quota manager executing operation");

        let (user, reserved) = self.reserve_quota(user_id, bytes).await?;

        match operation().await {
            Ok(result) => {
                let committed_event = user.commit_quota(&reserved);
                if let Err(e) = self
                    .event_bus
                    .publish(UserEvent::from(committed_event))
                    .await
                {
                    warn!(
                        user_id = %user_id,
                        bytes = %bytes.as_u64(),
                        error = %e,
                        "Failed to publish quota committed event"
                    );
                }
                info!(
                    bytes = %bytes.as_u64(),
                    "Quota operation completed successfully"
                );
                Ok(result)
            }
            Err(e) => {
                if let Err(rollback_err) = self.release_quota(user_id, reserved).await {
                    error!(
                        user_id = %user_id,
                        bytes = %bytes.as_u64(),
                        error = ?rollback_err,
                        "CRITICAL: Failed to rollback quota reservation. Manual intervention required"
                    );
                }
                Err(e)
            }
        }
    }

    #[instrument(skip(self), fields(user_id = %user_id, bytes = %bytes.as_u64()))]
    async fn reserve_quota(
        &self,
        user_id: UserId,
        bytes: Byte,
    ) -> ApplicationResult<(User, QuotaReservedEvent)> {
        debug!("Attempting quota reservation");

        let result = self
            .update_with_optimistic_locking(user_id, 5, |user| {
                let event = user
                    .reserve_quota(bytes)
                    .map_err(|e| ApplicationError::Domain { source: e })?;
                Ok((UserEvent::from(event.clone()), event))
            })
            .await?;

        info!(
            bytes_reserved = %bytes.as_u64(),
            "Quota reserved successfully"
        );

        Ok(result)
    }

    #[instrument(skip(self, reserved), fields(user_id = %user_id, bytes = %reserved.bytes.as_u64()))]
    async fn release_quota(
        &self,
        user_id: UserId,
        reserved: QuotaReservedEvent,
    ) -> ApplicationResult<User> {
        debug!("Releasing quota reservation");

        let result = self
            .update_with_optimistic_locking(user_id, 5, |user| {
                let event = user.release_quota(&reserved);
                Ok((UserEvent::from(event.clone()), event))
            })
            .await
            .map(|(user, _)| user)?;

        info!(
            bytes_released = %reserved.bytes.as_u64(),
            "Quota released successfully"
        );

        Ok(result)
    }

    /// Update user with optimistic locking. The closure returns a (UserEvent, T) tuple
    /// where UserEvent is published to the bus and T is returned to the caller.
    async fn update_with_optimistic_locking<T, F>(
        &self,
        user_id: UserId,
        retries: u32,
        f: F,
    ) -> ApplicationResult<(User, T)>
    where
        T: Clone,
        F: Fn(&mut User) -> ApplicationResult<(UserEvent, T)>,
    {
        let mut last_version = 0;

        for attempt in 0..retries {
            let mut user = self
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

            let (event, value) = f(&mut user)?;

            match self.user_repository.update(&user).await {
                Ok(_) => {
                    if let Err(e) = self.event_bus.publish(event).await {
                        warn!(
                            user_id = %user_id,
                            error = %e,
                            "Failed to publish user event"
                        );
                    }
                    return Ok((user, value));
                }
                Err(DomainError::ConcurrentModification { .. }) => {
                    if attempt < retries - 1 {
                        let backoff_ms = 10 * 2_u64.pow(attempt);
                        warn!(
                            user_id = %user_id,
                            attempt = attempt + 1,
                            max_retries = retries,
                            backoff_ms = backoff_ms,
                            version = last_version,
                            "Concurrent modification detected, retrying with backoff"
                        );
                        sleep(Duration::from_millis(backoff_ms)).await;
                    } else {
                        error!(
                            user_id = %user_id,
                            attempts = retries,
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
}
