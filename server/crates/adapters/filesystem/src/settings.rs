use std::path::PathBuf;

/// Plain settings for the filesystem storage adapter.
/// Mapped from the global config by the composition root.
#[derive(Debug, Clone)]
pub struct FilesystemSettings {
    pub base_path: PathBuf,
    pub tmp_path: PathBuf,
    pub cache_path: PathBuf,
}
