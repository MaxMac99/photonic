use crate::{
    medium::{
        model::{CreateMediumInput, MediumType},
        repo,
    },
    medium_item::{model::MediumItemType, service::create_medium_item},
    state::AppState,
    user::service::create_or_update_user,
};
use common::{error::Result, stream::events::StorageLocation, user::User};
use mime::Mime;
use mime_serde_shim::Wrapper;
use std::future::Future;
use uuid::Uuid;

pub async fn create_medium<F, Fut>(
    state: AppState,
    tmp_file: F,
    user: User,
    opts: CreateMediumInput,
    mime: Mime,
) -> Result<Uuid>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<StorageLocation>>,
{
    let user_id = user.sub;
    create_or_update_user(&state, user.clone().into()).await?;

    let medium_type = opts
        .medium_type
        .unwrap_or_else(|| MediumType::from(mime.clone()));
    let id = repo::create_medium(&state.db_pool, &user_id, medium_type).await?;

    create_medium_item(
        state,
        tmp_file,
        user,
        opts.into(),
        Wrapper(mime),
        id,
        MediumItemType::Original,
    )
    .await?;

    Ok(id)
}
