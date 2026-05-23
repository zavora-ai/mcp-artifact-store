use std::path::PathBuf;
use tokio::fs;

use crate::error::StoreError;
use crate::store::BlobStore;

/// Local filesystem blob storage.
pub struct LocalBlobStore {
    root: PathBuf,
}

impl LocalBlobStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn resolve(&self, uri: &str) -> PathBuf {
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        self.root.join(path)
    }
}

#[async_trait::async_trait]
impl BlobStore for LocalBlobStore {
    async fn health_check(&self) -> Result<(), StoreError> {
        fs::create_dir_all(&self.root).await.map_err(|e| StoreError::Storage(e.to_string()))
    }

    async fn put(&self, uri: &str, data: &[u8], _mime_type: &str) -> Result<(), StoreError> {
        let path = self.resolve(uri);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| StoreError::Storage(e.to_string()))?;
        }
        fs::write(&path, data).await.map_err(|e| StoreError::Storage(e.to_string()))
    }

    async fn get(&self, uri: &str) -> Result<Vec<u8>, StoreError> {
        let path = self.resolve(uri);
        fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::NotFound(uri.into())
            } else {
                StoreError::Storage(e.to_string())
            }
        })
    }

    async fn delete(&self, uri: &str) -> Result<(), StoreError> {
        let path = self.resolve(uri);
        fs::remove_file(&path).await.map_err(|e| StoreError::Storage(e.to_string()))
    }

    async fn exists(&self, uri: &str) -> Result<bool, StoreError> {
        Ok(self.resolve(uri).exists())
    }
}
