use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum EventSourcingError {
    #[snafu(display("Failed to serialize event: {source}"))]
    Serialization { source: serde_json::Error },

    #[snafu(display("Failed to deserialize event: {source}"))]
    Deserialization { source: serde_json::Error },

    #[snafu(display(
        "Concurrency conflict on stream '{stream_id}' at version {expected_version}"
    ))]
    ConcurrencyConflict {
        stream_id: String,
        expected_version: i64,
    },

    #[snafu(display("Event store error: {message}"))]
    Store { message: String },

    #[snafu(display("Projection error: {message}"))]
    Projection { message: String },

    #[snafu(display("Event bus error: {message}"))]
    Bus { message: String },

    #[snafu(display("Transaction error: {message}"))]
    Transaction { message: String },
}

pub type Result<T> = std::result::Result<T, EventSourcingError>;
