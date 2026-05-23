use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;

use crate::error::StoreError;
use crate::store::BlobStore;

/// S3 blob storage backend.
pub struct S3BlobStore {
    client: Client,
    bucket: String,
}

impl S3BlobStore {
    pub async fn new(bucket: String, region: Option<String>) -> Self {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(r) = region {
            config_loader = config_loader.region(aws_config::Region::new(r));
        }
        let config = config_loader.load().await;
        Self { client: Client::new(&config), bucket }
    }

    fn parse_key<'a>(&self, uri: &'a str) -> &'a str {
        uri.strip_prefix(&format!("s3://{}/", self.bucket))
            .or_else(|| uri.strip_prefix("s3://"))
            .unwrap_or(uri)
    }
}

#[async_trait::async_trait]
impl BlobStore for S3BlobStore {
    async fn health_check(&self) -> Result<(), StoreError> {
        self.client.head_bucket().bucket(&self.bucket).send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn put(&self, uri: &str, data: &[u8], mime_type: &str) -> Result<(), StoreError> {
        let key = self.parse_key(uri);
        self.client.put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(mime_type)
            .body(ByteStream::from(data.to_vec()))
            .send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, uri: &str) -> Result<Vec<u8>, StoreError> {
        let key = self.parse_key(uri);
        let resp = self.client.get_object()
            .bucket(&self.bucket)
            .key(key)
            .send().await
            .map_err(|e| StoreError::NotFound(e.to_string()))?;
        let bytes = resp.body.collect().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(bytes.to_vec())
    }

    async fn delete(&self, uri: &str) -> Result<(), StoreError> {
        let key = self.parse_key(uri);
        self.client.delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send().await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, uri: &str) -> Result<bool, StoreError> {
        let key = self.parse_key(uri);
        match self.client.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
