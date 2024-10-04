use crate::config::DatabaseConfig;
use snafu::{ResultExt, Whatever};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::future::Future;

pub async fn init_db<F, Fut>(config: &DatabaseConfig, migrate: F) -> Result<PgPool, Whatever>
where
    F: FnOnce(&PgPool) -> Fut,
    Fut: Future<Output = Result<(), Whatever>>,
{
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&config.url)
        .await
        .whatever_context("Could not create build pool")?;

    migrate(&pool).await?;

    pool.begin().await.unwrap();

    Ok(pool)
}
