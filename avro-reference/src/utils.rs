use apache_avro::{
    schema::{derive::AvroSchemaComponent, Name, Namespace},
    Schema,
};
use std::collections::HashMap;

pub struct TimestampMillis;

impl AvroSchemaComponent for TimestampMillis {
    fn get_schema_in_ctxt(_: &mut HashMap<Name, Schema>, _: &Namespace) -> Schema {
        Schema::TimestampMillis
    }
}
