use clap::{command, Command};

mod cli;

#[actix_web::main]
async fn main() -> Result<(), core::Error> {
    let matches = command!()
        .subcommand(Command::new(cli::SERVER_SUBCOMMAND)
            .about(cli::SERVER_DESCRIPTION))
        .get_matches();

    if let Some(_) = matches.subcommand_matches(cli::SERVER_SUBCOMMAND) {
        cli::server::run().await?;
    }

    Ok(())
}
