use clap::{Parser, Subcommand};
use snafu::Whatever;
use xtask::openapi::{convert_openapi, generate_openapi_spec};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation tasks for photonic")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate OpenAPI spec from utoipa definitions
    GenerateOpenapi {
        /// Output file path
        #[arg(short, long, default_value = "openapi.yaml")]
        output: String,
    },
    ConvertOpenapi {
        /// Input file path
        #[arg(short, long, default_value = "openapi.yaml")]
        input: String,
        /// Output file path
        #[arg(short, long, default_value = "openapi-3.0.yaml")]
        output: String,
    },
}

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateOpenapi { output } => generate_openapi_spec(&output).await,
        Commands::ConvertOpenapi { input, output } => convert_openapi(&input, &output),
    }
}
