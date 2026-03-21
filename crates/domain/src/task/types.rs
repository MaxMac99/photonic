use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    MetadataExtraction,
    TempCleanup,
}
