use crate::ReferenceSchema;
use apache_avro::{
    schema::{Name, Namespace},
    Schema,
};
use byte_unit::Byte;
use chrono::{DateTime, TimeZone};
use mime_serde_shim::Wrapper;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

pub trait AvroReferenceSchemaComponent {
    fn get_schema_in_ctx(
        named_schemas: &mut HashMap<Name, Schema>,
        enclosing_namespace: &Namespace,
    ) -> ReferenceSchema;
}

impl<T> crate::AvroReferenceSchema for T
where
    T: AvroReferenceSchemaComponent,
{
    fn get_reference_schema() -> ReferenceSchema {
        T::get_schema_in_ctx(&mut HashMap::default(), &None)
    }
}

impl<Tz: TimeZone> AvroReferenceSchemaComponent for DateTime<Tz> {
    fn get_schema_in_ctx(_: &mut HashMap<Name, Schema>, _: &Namespace) -> ReferenceSchema {
        Schema::String.into()
    }
}

impl<T> AvroReferenceSchemaComponent for Option<T>
where
    T: AvroReferenceSchemaComponent,
{
    fn get_schema_in_ctx(
        named_schemas: &mut HashMap<Name, Schema>,
        enclosing_namespace: &Namespace,
    ) -> ReferenceSchema {
        T::get_schema_in_ctx(named_schemas, enclosing_namespace)
    }
}

macro_rules! impl_schema (
    ($type:ty, $variant_constructor:expr) => (
        impl AvroReferenceSchemaComponent for $type {
            fn get_schema_in_ctx(_: &mut HashMap<Name, Schema>, _: &Namespace) -> ReferenceSchema {
                $variant_constructor.into()
            }
        }
    );
);

impl_schema!(String, Schema::String);
impl_schema!(i64, Schema::Long);
impl_schema!(Uuid, Schema::Uuid);
impl_schema!(PathBuf, Schema::String);
impl_schema!(Byte, Schema::Long);
impl_schema!(Wrapper, Schema::String);
