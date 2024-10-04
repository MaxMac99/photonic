use crate::{AvroReferenceSchemaComponent, ReferenceSchema};
use apache_avro::{
    schema::{Name, Namespace},
    Schema,
};
use std::collections::HashMap;

pub struct TimestampMillis;

impl AvroReferenceSchemaComponent for TimestampMillis {
    fn get_schema_in_ctx(_: &mut HashMap<Name, Schema>, _: &Namespace) -> ReferenceSchema {
        Schema::TimestampMillis.into()
    }
}
