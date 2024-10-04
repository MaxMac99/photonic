use crate::{user::model::UserClaims, Config};
use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use snafu::{ResultExt, Whatever};
use std::sync::Arc;

pub async fn create_auth(config: &Arc<Config>) -> Result<Authorizer<UserClaims>, Whatever> {
    let validation = Validation::new().aud(&[config.oauth.client_id.clone()]);
    JwtAuthorizer::from_jwks_url(&config.oauth.jwks_url)
        .validation(validation)
        .check(|user: &UserClaims| {
            user.given_name.is_some()
                || user.name.is_some()
                || user.nickname.is_some()
                || user.preferred_username.is_some()
        })
        .build()
        .await
        .whatever_context("Could not create JWT Authorizer")
        .into()
}
