use crate::ksqldb::types::{CommandState, Entity, Format};
use serde::Deserialize;

/// Generic information about the request
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct RequestInfo {
    #[serde(rename = "@type")]
    pub response_type: String,
    pub statement_text: Option<String>,
    pub warnings: Option<Vec<Warning>>,
}

/// The status of the current command being processed
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct CommandStatus {
    pub status: CommandState,
    pub message: String,
}

/// Warnings that were returned by KSQL
#[derive(Clone, Debug, Deserialize)]
pub struct Warning {
    pub message: String,
}

/// Information about a KSQL Stream
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct Stream {
    pub name: String,
    pub topic: String,
    pub format: Option<Format>,
    #[serde(rename = "type")]
    pub entity: Entity,
}

/// Information about a KSQL Table
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct Table {
    pub name: String,
    pub topic: String,
    pub format: Format,
    #[serde(rename = "type")]
    pub entity: Entity,
    pub is_windowed: bool,
}
