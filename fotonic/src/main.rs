use std::path::PathBuf;

use clap::{command, Command};
use env_logger::{Env, Target};
use log::debug;

mod cli;

#[tokio::main]
async fn main() -> Result<(), core::Error> {
    dotenv::dotenv()
        .map_err(|err| core::Error::Internal(format!("Could not read dot env: {}", err.to_string())))?;

    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    builder.target(Target::Stdout);
    builder.init();

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
        exiftool.read_file(target_path).await.expect("Error reading file");
    }

    Ok(())
}
