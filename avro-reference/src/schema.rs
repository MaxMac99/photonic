use apache_avro::Schema;

#[derive(Debug, Clone)]
pub struct ReferenceSchema {
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

impl From<Schema> for ReferenceSchema {
    fn from(value: Schema) -> Self {
        ReferenceSchema {
            schema: value,
            references: vec![],
        }
    }
}
