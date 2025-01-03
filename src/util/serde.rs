use byte_unit::Byte;
use serde::Serializer;

pub fn serialize_byte_as_u64<S>(byte: &Byte, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(byte.as_u64())
}
