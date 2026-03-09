use std::sync::Arc;

use byte_unit::Byte;
use derive_new::new;
use tracing::{debug, info, warn};

use crate::{
    application::{
        error::ApplicationResult,
        user::ports::{PublishUserEvent, UserRepository},
    },
    domain::user::{User, UserCreateRequest, UserId, UserUpdateRequestBuilder},
    infrastructure::config::GlobalConfig,
};

pub struct EnsureUserExistsCommand {
    pub user_id: UserId,
    pub username: String,
    pub email: Option<String>,
    pub quota: Option<Byte>,
}

#[derive(new)]
pub struct EnsureUserExistsHandler {
    user_repository: Arc<dyn UserRepository>,
    event_bus: Arc<dyn PublishUserEvent>,
    config: Arc<GlobalConfig>,
}

impl EnsureUserExistsHandler {
    pub async fn handle(&self, command: EnsureUserExistsCommand) -> ApplicationResult<UserId> {
        debug!(
            "EnsureUserExists: Checking user_id={}, username={}",
            command.user_id, command.username
        );

        let quota_limit = command
            .quota
            .unwrap_or(Byte::from_u64(self.config.storage().default_user_quota));

        let max_quota = Byte::from_u64(self.config.storage().max_user_quota);

        match self.user_repository.find_by_id(command.user_id).await? {
            Some(mut existing_user) => {
                debug!(
                    "User {} already exists in database: username={}, email={:?}",
                    command.user_id, existing_user.username, existing_user.email
                );

                let update_request = UserUpdateRequestBuilder::default()
                    .username(command.username.clone())
                    .email(command.email.clone())
                    .quota(quota_limit)
                    .build()
                    .unwrap();

                if let Some(event) = existing_user.update(update_request, max_quota)? {
                    info!(
                        "User {} updated: username={}, email={:?}, quota={}",
                        existing_user.id, command.username, command.email, quota_limit
                    );

                    self.user_repository.update(&existing_user).await?;
                    debug!("User {} changes persisted to database", existing_user.id);

                    if let Err(e) = self.event_bus.publish(event).await {
                        warn!(
                            "Failed to publish event for user {}: {:?}",
                            existing_user.id, e
                        );
                    } else {
                        debug!("Published UserUpdated event for user {}", existing_user.id);
                    }
                } else {
                    debug!("User {} unchanged - no update needed", existing_user.id);
                }

                Ok(existing_user.id)
            }
            None => {
                debug!(
                    "Creating new user: user_id={}, username={}, email={:?}, quota={}",
                    command.user_id, command.username, command.email, quota_limit
                );

                let create_request = UserCreateRequest {
                    sub: command.user_id,
                    username: command.username.clone(),
                    email: command.email.clone(),
                    quota: quota_limit,
                };

                let (user, event) = User::new(create_request, max_quota)?;

                self.user_repository.insert(&user).await?;
                debug!("New user {} persisted to database", user.id);

                if let Err(e) = self.event_bus.publish(event).await {
                    warn!(
                        "Failed to publish user created event for user {}: {:?}",
                        user.id, e
                    );
                } else {
                    debug!("Published UserCreated event for user {}", user.id);
                }

                info!(
                    "Successfully created user {} (username: {}, email: {:?}, quota: {})",
                    user.id, command.username, command.email, quota_limit
                );

                Ok(user.id)
            }
        }
    }
}
