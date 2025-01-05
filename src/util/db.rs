use crate::{
    config::DatabaseConfig,
    state::{AppState, ArcConnection},
};
use futures_util::future::BoxFuture;
use snafu::{ResultExt, Whatever};
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn init_db(config: &DatabaseConfig) -> Result<PgPool, Whatever> {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&config.url)
        .await
        .whatever_context("Could not create build pool")?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .whatever_context("Could not run migrations")?;

    pool.begin().await.unwrap();

    Ok(pool)
}

pub async fn run_with_transaction<F, O>(state: AppState, f: F) -> crate::error::Result<O>
where
    F: FnOnce(AppState, ArcConnection) -> BoxFuture<'_, crate::error::Result<O>>,
{
    let mut transaction = state.begin_transaction().await?;
    let result = f(state, ArcConnection::new(&mut transaction)).await?;
    transaction.commit().await?;
    Ok(result)
}
