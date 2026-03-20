/// SHA-256 checksum (32 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sha256([u8; 32]);

impl Sha256 {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

/// Utility functions for hashing
pub mod hash {
    use sha2::{Digest, Sha256 as Sha256Hasher};
    use tokio::io::{AsyncRead, AsyncReadExt};

    /// Calculate SHA-256 hash from bytes
    pub fn sha256_bytes(data: &[u8]) -> super::Sha256 {
        let mut hasher = Sha256Hasher::new();
        hasher.update(data);
        let hash_bytes: [u8; 32] = hasher.finalize().into();
        super::Sha256::new(hash_bytes)
    }

    /// Calculate SHA-256 hash from an async reader
    pub async fn sha256_stream<R: AsyncRead + Unpin>(
        mut reader: R,
    ) -> std::io::Result<super::Sha256> {
        let mut hasher = Sha256Hasher::new();
        let mut buffer = [0; 8192];

        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let hash_bytes: [u8; 32] = hasher.finalize().into();
        Ok(super::Sha256::new(hash_bytes))
    }

    /// Calculate SHA-256 hash from a file
    pub async fn sha256_file(path: &std::path::Path) -> std::io::Result<super::Sha256> {
        let file = tokio::fs::File::open(path).await?;
        sha256_stream(file).await
    }
}
