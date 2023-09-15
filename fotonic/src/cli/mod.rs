pub mod server;

pub const SERVER_SUBCOMMAND: &str = "server";
pub const SERVER_DESCRIPTION: &str = "Run the http_server";

fn init_logger() {
    env_logger::init();
}
