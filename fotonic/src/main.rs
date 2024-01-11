use std::path::PathBuf;

use clap::{command, Command};
use tracing::log::debug;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use exiftool::ExifError;

mod cli;

#[tokio::main]
async fn main() -> Result<(), core::Error> {
    dotenv::dotenv()
        .map_err(|err| core::Error::Internal(format!("Could not read dot env: {}", err.to_string())))?;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let matches = command!()
        .subcommand(Command::new(cli::SERVER_SUBCOMMAND)
            .about(cli::SERVER_DESCRIPTION))
        .subcommand(Command::new("exif"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches(cli::SERVER_SUBCOMMAND) {
        cli::server::run().await?;
    }
    if let Some(_) = matches.subcommand_matches("exif") {
        let exiftool = exiftool::Exiftool::new().await.unwrap();
        let target_path = std::env::current_dir().unwrap().join(PathBuf::from("test/IMG_4597.DNG"));
        debug!("Target: {}", target_path.display());
        exiftool.read_file(target_path, false, false).await
            .map_err(|err| <ExifError as Into<meta::Error>>::into(err))?;
    }

    Ok(())
}
