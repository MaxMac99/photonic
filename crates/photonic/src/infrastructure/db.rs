use snafu::{ResultExt, Whatever};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::infrastructure::config::DatabaseConfig;

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
