use apache_avro::{schema::Name, Schema};

#[derive(Debug, Clone)]
pub struct ReferenceSchema {
    pub name: Name,
    pub schema: Schema,
    pub references: Vec<ReferenceSchema>,
}

pub trait AvroReferenceSchema {
    fn get_reference_schema() -> ReferenceSchema;
}

impl Into<Schema> for ReferenceSchema {
    fn into(self) -> Schema {
        self.schema
    }
}
