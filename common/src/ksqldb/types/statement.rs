use crate::ksqldb::types::common::{CommandStatus, RequestInfo, Stream, Table};
use serde::Deserialize;

/// The response type for any `CREATE` KSQL-DB Statement
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct CreateResponse {
    #[serde(flatten)]
    pub info: RequestInfo,

    // Statement specific fields
    pub command_id: Option<String>,
    pub command_status: Option<CommandStatus>,
    pub command_sequence_number: Option<i32>,
}

/// The response type for any `LIST STREAMS` KSQL-DB Statement
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct ListStreamsResponse {
    #[serde(flatten)]
    pub info: RequestInfo,

    // Statement specific fields
    pub source_descriptions: Vec<Stream>,
}

/// The response type for any `LIST TABLES` KSQL-DB Statement
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
pub struct ListTablesResponse {
    #[serde(flatten)]
    pub info: RequestInfo,

    // Statement specific fields
    pub tables: Vec<Table>,
}
