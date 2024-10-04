pub(super) mod error;
pub mod statement;
mod stream;
pub mod types;

use reqwest::Client;

pub use error::{Error, Result};

pub struct KsqlDb {
    url: String,
    client: Client,
}

impl KsqlDb {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }
}
