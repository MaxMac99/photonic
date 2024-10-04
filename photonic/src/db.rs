use std::sync::Arc;

use snafu::{ResultExt, Whatever};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::Config;

pub async fn init_db(config: Arc<Config>) -> Result<PgPool, Whatever> {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&config.database.url)
        .await
        .whatever_context("Could not create build pool")?;

    run_migrations(&pool).await?;

    pool.begin().await.unwrap();

    Ok(pool)
}

async fn run_migrations(pool: &PgPool) -> Result<(), Whatever> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .whatever_context("Could not run migrations")
}
