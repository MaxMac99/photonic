use application::user::commands::EnsureUserExistsCommand;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use domain::error::ValidationSnafu;
use jwt_authorizer::JwtClaims;
use snafu::OptionExt;
use tracing::{debug, trace};

use crate::{
    api::{error::ApiResult, state::AppState},
    auth::JwtUserClaims,
};

/// Middleware that ensures authenticated users exist in our database
/// This runs after JWT validation but before the actual handler
pub async fn ensure_user_exists(
    State(state): State<AppState>,
    claims: Option<JwtClaims<JwtUserClaims>>,
    request: Request,
    next: Next,
) -> ApiResult<Response> {
    // Only process if we have valid JWT claims
    if let Some(JwtClaims(user_claims)) = claims {
        let user_id = user_claims.user_id();
        trace!("Processing authenticated request for user_id={}", user_id);

        // Create command for the application layer
        let command = EnsureUserExistsCommand {
            user_id,
            username: user_claims.get_username().context(ValidationSnafu {
                message: "No valid username found in JWT claims",
            })?,
            email: user_claims.email.clone(),
            quota: user_claims.quota,
        };

        debug!(
            "Ensuring user exists: user_id={}, username={}, email={:?}, quota={:?}",
            user_id, command.username, user_claims.email, user_claims.quota
        );

        state.user_handlers.user_exists.handle(command).await?;
    } else {
        trace!("No JWT claims present - skipping user existence check");
    }

    // Continue to the actual handler
    Ok(next.run(request).await)
}
