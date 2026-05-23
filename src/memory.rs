use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::StoreError;
use crate::store::MetadataStore;
use crate::types::*;

/// In-memory metadata store for development and testing.
pub struct MemoryMetadataStore {
    artifacts: Arc<RwLock<HashMap<String, Artifact>>>,
    versions: Arc<RwLock<HashMap<String, ArtifactVersion>>>,
    edges: Arc<RwLock<Vec<ArtifactEdge>>>,
}

impl MemoryMetadataStore {
    pub fn new() -> Self {
        Self {
            artifacts: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl MetadataStore for MemoryMetadataStore {
    async fn list_folders(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let arts = self.artifacts.read().await;
        let mut folders: Vec<String> = arts
            .values()
            .map(|a| a.folder_path.clone())
            .filter(|p| p.starts_with(prefix))
            .collect();
        folders.sort();
        folders.dedup();
        Ok(folders)
    }

    async fn list_artifacts(&self, folder: Option<&str>, session_id: Option<&str>, class: Option<&ArtifactClass>) -> Result<Vec<Artifact>, StoreError> {
        let arts = self.artifacts.read().await;
        let results = arts.values().filter(|a| {
            folder.is_none_or(|f| a.folder_path.starts_with(f))
                && session_id.is_none_or(|s| a.session_id.as_deref() == Some(s))
                && class.is_none_or(|c| &a.artifact_class == c)
        }).cloned().collect();
        Ok(results)
    }

    async fn get_artifact(&self, id: &str) -> Result<Artifact, StoreError> {
        self.artifacts.read().await.get(id).cloned().ok_or_else(|| StoreError::NotFound(id.into()))
    }

    async fn put_artifact(&self, artifact: Artifact) -> Result<(), StoreError> {
        self.artifacts.write().await.insert(artifact.artifact_id.clone(), artifact);
        Ok(())
    }

    async fn update_artifact(&self, artifact: Artifact) -> Result<(), StoreError> {
        let mut arts = self.artifacts.write().await;
        if !arts.contains_key(&artifact.artifact_id) {
            return Err(StoreError::NotFound(artifact.artifact_id));
        }
        arts.insert(artifact.artifact_id.clone(), artifact);
        Ok(())
    }

    async fn delete_artifact(&self, id: &str) -> Result<(), StoreError> {
        self.artifacts.write().await.remove(id).ok_or_else(|| StoreError::NotFound(id.into()))?;
        Ok(())
    }

    async fn get_version(&self, version_id: &str) -> Result<ArtifactVersion, StoreError> {
        self.versions.read().await.get(version_id).cloned().ok_or_else(|| StoreError::NotFound(version_id.into()))
    }

    async fn get_current_version(&self, artifact_id: &str) -> Result<Option<ArtifactVersion>, StoreError> {
        let art = self.get_artifact(artifact_id).await?;
        match art.current_version_id {
            Some(vid) => Ok(Some(self.get_version(&vid).await?)),
            None => Ok(None),
        }
    }

    async fn put_version(&self, version: ArtifactVersion) -> Result<(), StoreError> {
        self.versions.write().await.insert(version.version_id.clone(), version);
        Ok(())
    }

    async fn put_edge(&self, edge: ArtifactEdge) -> Result<(), StoreError> {
        self.edges.write().await.push(edge);
        Ok(())
    }

    async fn get_edges_from(&self, artifact_id: &str) -> Result<Vec<ArtifactEdge>, StoreError> {
        let edges = self.edges.read().await;
        Ok(edges.iter().filter(|e| e.from_artifact_id == artifact_id).cloned().collect())
    }

    async fn get_edges_to(&self, artifact_id: &str) -> Result<Vec<ArtifactEdge>, StoreError> {
        let edges = self.edges.read().await;
        Ok(edges.iter().filter(|e| e.to_artifact_id == artifact_id).cloned().collect())
    }
}
