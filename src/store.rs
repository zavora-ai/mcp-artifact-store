use crate::error::StoreError;
use crate::types::*;

/// Metadata registry — stores artifact records, versions, edges.
#[async_trait::async_trait]
pub trait MetadataStore: Send + Sync {
    async fn list_folders(&self, prefix: &str) -> Result<Vec<String>, StoreError>;
    async fn list_artifacts(&self, folder: Option<&str>, session_id: Option<&str>, class: Option<&ArtifactClass>) -> Result<Vec<Artifact>, StoreError>;
    async fn get_artifact(&self, id: &str) -> Result<Artifact, StoreError>;
    async fn put_artifact(&self, artifact: Artifact) -> Result<(), StoreError>;
    async fn update_artifact(&self, artifact: Artifact) -> Result<(), StoreError>;
    async fn delete_artifact(&self, id: &str) -> Result<(), StoreError>;

    async fn get_version(&self, version_id: &str) -> Result<ArtifactVersion, StoreError>;
    async fn get_current_version(&self, artifact_id: &str) -> Result<Option<ArtifactVersion>, StoreError>;
    async fn put_version(&self, version: ArtifactVersion) -> Result<(), StoreError>;

    async fn put_edge(&self, edge: ArtifactEdge) -> Result<(), StoreError>;
    async fn get_edges_from(&self, artifact_id: &str) -> Result<Vec<ArtifactEdge>, StoreError>;
    async fn get_edges_to(&self, artifact_id: &str) -> Result<Vec<ArtifactEdge>, StoreError>;
}

/// Blob storage — stores raw artifact content.
#[async_trait::async_trait]
pub trait BlobStore: Send + Sync {
    async fn health_check(&self) -> Result<(), StoreError>;
    async fn put(&self, uri: &str, data: &[u8], mime_type: &str) -> Result<(), StoreError>;
    async fn get(&self, uri: &str) -> Result<Vec<u8>, StoreError>;
    async fn delete(&self, uri: &str) -> Result<(), StoreError>;
    async fn exists(&self, uri: &str) -> Result<bool, StoreError>;
}
