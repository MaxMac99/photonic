use std::sync::Arc;

use deadpool_diesel::{
    postgres::{Manager, Pool},
    Runtime::Tokio1,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use snafu::{ResultExt, Whatever};

use crate::config::Config;

mod album;
mod dto;
mod medium;
mod to_trash;
mod user;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub struct Repository {
    pool: Pool,
}

impl Repository {
    pub async fn init(config: Arc<Config>) -> Result<Self, Whatever> {
        let manager = Manager::new(&config.database.url, Tokio1);
        let pool = Pool::builder(manager)
            .build()
            .whatever_context("Could not create build pool")?;

        run_migrations(&pool).await?;

        Ok(Self { pool })
    }
}

async fn run_migrations(pool: &Pool) -> Result<(), Whatever> {
    let conn = pool
        .get()
        .await
        .whatever_context("Could not get connection")?;
    let _ = conn
        .interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .whatever_context("Could not run migrations")?;
    Ok(())
}
