use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, AvroSchema)]
pub struct MediumCreatedEvent {
    pub id: Uuid,
    pub path: String,
}
