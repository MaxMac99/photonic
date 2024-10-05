mod created_event;
mod exif_loaded_event;

pub use created_event::*;
pub use exif_loaded_event::*;

use avro_reference::AvroReferenceSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, AvroReferenceSchema,
)]
#[sqlx(type_name = "medium_item_type_enum", rename_all = "lowercase")]
#[avro(referencable, namespace = "de.vissing.photonic")]
pub enum MediumItemType {
    Original,
    Edit,
    Preview,
}
