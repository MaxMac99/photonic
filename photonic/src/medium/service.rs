use crate::{
    error::Result,
    medium::model::Medium,
    server::AppState,
    service::CreateUserInput,
    store::{service::store_stream_temporarily, Transaction},
    user::{model::CreateUserInput, repo::create_or_update_user, service::CreateUserInput},
    AppState,
};
use axum::body::Bytes;
use chrono::{DateTime, FixedOffset};
use futures::Stream;
use mime::Mime;
use redis::AsyncCommands;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateMediumInput {
    pub user: CreateUserInput,
    pub album_id: Option<Uuid>,
    pub filename: String,
    pub extension: String,
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub mime: Mime,
    pub priority: i32,
}

pub async fn create_medium<S, E>(
    app_state: AppState,
    input: CreateMediumInput,
    stream: S,
) -> Result<Uuid>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
{
    create_or_update_user(&app_state.db_pool, input.user).await?;

    let mut transaction = Transaction::begin();
    let path = store_stream_temporarily(
        &mut transaction,
        &app_state.config,
        &input.extension,
        stream,
    )
    .await?;

    let id = Uuid::new_v4();
    let mut con = app_state.cache_pool.get().await?;
    con.hset_multiple(
        format!("medium:{}", id),
        &[("path", path.into_os_string().into_string()?)],
    );
    transaction.commit();
    Ok(id)
}

pub async fn get_medium_info(app_state: AppState, id: Uuid) -> Result<Medium> {}
