use snafu::{ResultExt, Whatever};
use sqlx::PgPool;

mod config;
mod state;
mod storage;

fn main() {
    println!("Hello, world!");
}

async fn run_migrations(pool: &PgPool) -> Result<(), Whatever> {
    sqlx::migrate!()
        .run(pool)
        .await
        .whatever_context("Could not run migrations")
}
