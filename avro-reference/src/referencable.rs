use crate::ReferenceSchema;
use apache_avro::{schema::UnionSchema, Schema};

pub trait AvroReferencable {
    fn get_referenced_schema() -> ReferenceSchema;
}

impl<T> AvroReferencable for Option<T>
where
    T: AvroReferencable,
{
    fn get_referenced_schema() -> ReferenceSchema {
        let inner = T::get_referenced_schema();
        ReferenceSchema {
            name: inner.name,
            schema: Schema::Union(UnionSchema::new(vec![Schema::Null, inner.schema]).unwrap()),
            references: inner.references,
        }
    }
}
