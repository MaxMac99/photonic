use std::path::PathBuf;

use clap::{command, Command};
use snafu::{ResultExt, Whatever};
use tracing::log::debug;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

mod cli;

#[tokio::main]
async fn main() -> Result<(), Whatever> {
    dotenv::dotenv().whatever_context("Could not initialize dotenv")?;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let matches = command!()
        .subcommand(
            Command::new(cli::SERVER_SUBCOMMAND).about(cli::SERVER_DESCRIPTION),
        )
        .subcommand(Command::new("exif"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches(cli::SERVER_SUBCOMMAND) {
        cli::server::run().await?;
    }
    if let Some(_) = matches.subcommand_matches("exif") {
        let meta = meta::Service::new()
            .await
            .whatever_context("Could not create meta service")?;
        let target_path = std::env::current_dir()
            .unwrap()
            .join(PathBuf::from("test/data/IMG_4597.DNG"));
        debug!("Target: {:?}", target_path);
        let exif = meta
            .read_file(target_path, true)
            .await
            .whatever_context("Could not read exif from file")?;
        debug!("Exif: {:#?}", exif);
    }

    Ok(())
}
