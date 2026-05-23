use std::sync::Arc;
use reqwest::Client;

use crate::error::StoreError;
use crate::store::BlobStore;

/// GCS blob storage backend.
pub struct GcsBlobStore {
    client: Client,
    bucket: String,
    token_provider: Arc<dyn gcp_auth::TokenProvider>,
}

impl GcsBlobStore {
    pub async fn new(bucket: String) -> Result<Self, StoreError> {
        let provider = gcp_auth::provider().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(Self { client: Client::new(), bucket, token_provider: provider })
    }

    async fn token(&self) -> Result<String, StoreError> {
        let scopes = &["https://www.googleapis.com/auth/devstorage.read_write"];
        let t = self.token_provider.token(scopes).await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(t.as_str().to_string())
    }

    fn parse_key<'a>(&self, uri: &'a str) -> &'a str {
        uri.strip_prefix(&format!("gs://{}/", self.bucket))
            .or_else(|| uri.strip_prefix("gs://"))
            .unwrap_or(uri)
    }

    fn upload_url(&self, key: &str) -> String {
        format!(
            "https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}",
            self.bucket,
            urlencoding::encode(key)
        )
    }

    fn object_url(&self, key: &str) -> String {
        format!(
            "https://storage.googleapis.com/storage/v1/b/{}/o/{}",
            self.bucket,
            urlencoding::encode(key)
        )
    }

    fn media_url(&self, key: &str) -> String {
        format!("{}?alt=media", self.object_url(key))
    }
}

#[async_trait::async_trait]
impl BlobStore for GcsBlobStore {
    async fn health_check(&self) -> Result<(), StoreError> {
        let token = self.token().await?;
        let url = format!("https://storage.googleapis.com/storage/v1/b/{}", self.bucket);
        let resp = self.client.get(&url).bearer_auth(&token).send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        if resp.status().is_success() { Ok(()) }
        else { Err(StoreError::Storage(format!("status: {}", resp.status()))) }
    }

    async fn put(&self, uri: &str, data: &[u8], mime_type: &str) -> Result<(), StoreError> {
        let key = self.parse_key(uri);
        let token = self.token().await?;
        let resp = self.client.post(&self.upload_url(key))
            .bearer_auth(&token)
            .header("Content-Type", mime_type)
            .body(data.to_vec())
            .send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        if resp.status().is_success() { Ok(()) }
        else { Err(StoreError::Storage(format!("upload failed: {}", resp.status()))) }
    }

    async fn get(&self, uri: &str) -> Result<Vec<u8>, StoreError> {
        let key = self.parse_key(uri);
        let token = self.token().await?;
        let resp = self.client.get(&self.media_url(key))
            .bearer_auth(&token)
            .send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        if resp.status() == 404 { return Err(StoreError::NotFound(uri.into())); }
        let bytes = resp.bytes().await.map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(bytes.to_vec())
    }

    async fn delete(&self, uri: &str) -> Result<(), StoreError> {
        let key = self.parse_key(uri);
        let token = self.token().await?;
        self.client.delete(&self.object_url(key))
            .bearer_auth(&token)
            .send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, uri: &str) -> Result<bool, StoreError> {
        let key = self.parse_key(uri);
        let token = self.token().await?;
        let resp = self.client.get(&self.object_url(key))
            .bearer_auth(&token)
            .send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(resp.status().is_success())
    }
}
