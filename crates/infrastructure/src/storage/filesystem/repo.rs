use std::{path::PathBuf, sync::Arc};

use application::medium::ports::FileStorage;
use async_trait::async_trait;
use domain::{
    error::{DomainResult, FileNotExistsSnafu},
    medium::storage::{FileLocation, FileMetadata, StorageTier},
    shared::crypto::hash,
};
use snafu::ensure;
use tokio::{
    fs,
    io::{AsyncRead, AsyncWriteExt},
};
use tracing::{debug, error, info};

use crate::config::GlobalConfig;

/// Filesystem implementation of FileStorage port
pub struct FilesystemStorageAdapter {
    config: Arc<GlobalConfig>,
}

impl FilesystemStorageAdapter {
    pub fn new(config: Arc<GlobalConfig>) -> Self {
        Self { config }
    }

    fn get_full_path(&self, location: &FileLocation) -> PathBuf {
        let base = match location.storage_tier {
            StorageTier::Permanent => &self.config.storage.base_path,
            StorageTier::Temporary => &self.config.storage.tmp_path,
            StorageTier::Cache => &self.config.storage.cache_path,
        };

        base.join(&location.relative_path)
    }
}

#[async_trait]
impl FileStorage for FilesystemStorageAdapter {
    #[tracing::instrument(skip(self, content), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path,
        size_bytes = content.len()
    ))]
    async fn store_file(&self, location: &FileLocation, content: Vec<u8>) -> DomainResult<()> {
        debug!("Starting file storage");

        let size = content.len();
        let full_path = self.get_full_path(location);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error!(path = ?parent, error = ?e, "Failed to create parent directories");
                e
            })?;
        }

        fs::write(&full_path, content).await.map_err(|e| {
            error!(path = ?full_path, error = ?e, "Failed to write file");
            e
        })?;

        info!(
            path = ?location.relative_path,
            size_bytes = size,
            storage_tier = ?location.storage_tier,
            "File stored successfully"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, stream), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path
    ))]
    async fn store_file_stream(
        &self,
        location: &FileLocation,
        mut stream: Box<dyn AsyncRead + Send + Unpin>,
    ) -> DomainResult<()> {
        debug!("Starting file stream storage");

        let full_path = self.get_full_path(location);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error!(path = ?parent, error = ?e, "Failed to create parent directories");
                e
            })?;
        }

        // Open file for writing
        let mut file = fs::File::create(&full_path).await.map_err(|e| {
            error!(path = ?full_path, error = ?e, "Failed to create file");
            e
        })?;

        // Stream the content to the file
        let bytes_written = tokio::io::copy(&mut stream, &mut file).await.map_err(|e| {
            error!(path = ?full_path, error = ?e, "Failed to copy stream to file");
            e
        })?;

        file.flush().await.map_err(|e| {
            error!(path = ?full_path, error = ?e, "Failed to flush file");
            e
        })?;

        info!(
            path = ?location.relative_path,
            size_bytes = bytes_written,
            storage_tier = ?location.storage_tier,
            "File stream stored successfully"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(
        src_tier = ?src.storage_tier,
        src_path = ?src.relative_path,
        dest_tier = ?dest.storage_tier,
        dest_path = ?dest.relative_path
    ))]
    async fn copy_file(&self, src: &FileLocation, dest: &FileLocation) -> DomainResult<()> {
        debug!("Copying file");

        let src_path = self.get_full_path(src);
        let dest_path = self.get_full_path(dest);

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error!(path = ?parent, error = ?e, "Failed to create parent directories for copy");
                e
            })?;
        }

        let bytes_copied = fs::copy(&src_path, &dest_path).await.map_err(|e| {
            error!(src = ?src_path, dest = ?dest_path, error = ?e, "Failed to copy file");
            e
        })?;

        info!(
            src_path = ?src.relative_path,
            dest_path = ?dest.relative_path,
            size_bytes = bytes_copied,
            src_tier = ?src.storage_tier,
            dest_tier = ?dest.storage_tier,
            "File copied successfully"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(
        src_tier = ?src.storage_tier,
        src_path = ?src.relative_path,
        dest_tier = ?dest.storage_tier,
        dest_path = ?dest.relative_path
    ))]
    async fn move_file(&self, src: &FileLocation, dest: &FileLocation) -> DomainResult<()> {
        debug!("Moving file");

        let src_path = self.get_full_path(src);
        let dest_path = self.get_full_path(dest);

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error!(path = ?parent, error = ?e, "Failed to create parent directories for move");
                e
            })?;
        }

        fs::rename(&src_path, &dest_path).await.map_err(|e| {
            error!(src = ?src_path, dest = ?dest_path, error = ?e, "Failed to move file");
            e
        })?;

        info!(
            src_path = ?src.relative_path,
            dest_path = ?dest.relative_path,
            src_tier = ?src.storage_tier,
            dest_tier = ?dest.storage_tier,
            "File moved successfully"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path
    ))]
    async fn retrieve_file(&self, location: &FileLocation) -> DomainResult<Vec<u8>> {
        debug!("Retrieving file");

        let path = self.get_full_path(location);

        let content = fs::read(&path).await.map_err(|e| {
            error!(path = ?path, error = ?e, "Failed to read file");
            e
        })?;

        debug!(
            size_bytes = content.len(),
            storage_tier = ?location.storage_tier,
            "File retrieved successfully"
        );
        Ok(content)
    }

    #[tracing::instrument(skip(self), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path
    ))]
    async fn retrieve_file_stream(
        &self,
        location: &FileLocation,
    ) -> DomainResult<Box<dyn AsyncRead + Unpin>> {
        debug!("Opening file stream for retrieval");

        let path = self.get_full_path(location);

        let file = fs::File::open(&path).await.map_err(|e| {
            error!(path = ?path, error = ?e, "Failed to open file");
            e
        })?;

        debug!(storage_tier = ?location.storage_tier, "File stream opened successfully");
        Ok(Box::new(file))
    }

    #[tracing::instrument(skip(self), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path
    ))]
    async fn get_local_path(&self, location: &FileLocation) -> DomainResult<PathBuf> {
        debug!("Getting local file path");

        let path = self.get_full_path(location);

        ensure!(path.exists(), FileNotExistsSnafu { path: path.clone() });

        debug!(
            full_path = ?path,
            storage_tier = ?location.storage_tier,
            "Local path retrieved"
        );
        Ok(path)
    }

    #[tracing::instrument(skip(self), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path
    ))]
    async fn delete_file(&self, location: &FileLocation) -> DomainResult<()> {
        debug!("Deleting file");

        let path = self.get_full_path(location);

        fs::remove_file(&path).await.map_err(|e| {
            error!(path = ?path, error = ?e, "Failed to delete file");
            e
        })?;

        info!(
            path = ?location.relative_path,
            storage_tier = ?location.storage_tier,
            "File deleted successfully"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(
        storage_tier = ?location.storage_tier,
        path = ?location.relative_path
    ))]
    async fn get_file_metadata(&self, location: &FileLocation) -> DomainResult<FileMetadata> {
        debug!("Getting file metadata");

        let path = self.get_full_path(location);

        let metadata = fs::metadata(&path).await.map_err(|e| {
            error!(path = ?path, error = ?e, "Failed to get file metadata");
            e
        })?;

        let checksum = hash::sha256_file(&path).await.map_err(|e| {
            error!(path = ?path, error = ?e, "Failed to calculate file checksum");
            e
        })?;

        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();

        info!(
            path = ?location.relative_path,
            size_bytes = metadata.len(),
            mime_type = %mime_type,
            storage_tier = ?location.storage_tier,
            "File metadata retrieved successfully"
        );

        Ok(FileMetadata {
            size_bytes: metadata.len(),
            mime_type,
            checksum,
        })
    }
}
