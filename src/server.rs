use std::sync::Arc;
use chrono::Utc;
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;

use crate::error::StoreError;
use crate::store::{BlobStore, MetadataStore};
use crate::types::*;

#[derive(Clone)]
pub struct ArtifactStoreServer {
    metadata: Arc<dyn MetadataStore>,
    blobs: Arc<dyn BlobStore>,
}

impl ArtifactStoreServer {
    pub fn new(metadata: Arc<dyn MetadataStore>, blobs: Arc<dyn BlobStore>) -> Self {
        Self { metadata, blobs }
    }

    fn make_uri(&self, artifact_id: &str, version: u32) -> String {
        format!("{}/v{}/blob", artifact_id, version)
    }
}

// --- Input types ---

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListFoldersInput { #[serde(default)] pub prefix: Option<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListArtifactsInput {
    #[serde(default)] pub folder: Option<String>,
    #[serde(default)] pub session_id: Option<String>,
    #[serde(default)] pub artifact_class: Option<ArtifactClass>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetArtifactMetadataInput { pub artifact_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReadArtifactInput { pub artifact_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct WriteArtifactInput {
    pub name: String,
    pub folder_path: String,
    pub content_base64: String,
    pub mime_type: String,
    #[serde(default)] pub artifact_class: Option<ArtifactClass>,
    #[serde(default)] pub retention_class: Option<RetentionClass>,
    #[serde(default)] pub session_id: Option<String>,
    #[serde(default)] pub owner: Option<String>,
    #[serde(default)] pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CreateVersionInput {
    pub artifact_id: String,
    pub content_base64: String,
    pub mime_type: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct RedactArtifactInput {
    pub artifact_id: String,
    pub redacted_content_base64: String,
    pub mime_type: String,
    #[serde(default)] pub fields_removed: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct DeriveArtifactInput {
    pub source_artifact_id: String,
    pub name: String,
    pub content_base64: String,
    pub mime_type: String,
    #[serde(default)] pub edge_type: Option<EdgeType>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct LinkArtifactsInput {
    pub from_artifact_id: String,
    pub to_artifact_id: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ExportPackageInput { pub artifact_ids: Vec<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct VerifyIntegrityInput { pub artifact_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetLineageInput { pub artifact_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SetRetentionInput { pub artifact_id: String, pub retention_class: RetentionClass }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct RequestAccessInput { pub artifact_id: String, pub reason: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct DeleteArtifactInput { pub artifact_id: String }

// --- Tool implementations ---

#[tool_router(server_handler)]
impl ArtifactStoreServer {
    #[tool(description = "Browse workspace folder hierarchy")]
    async fn list_folders(&self, Parameters(i): Parameters<ListFoldersInput>) -> String {
        match self.metadata.list_folders(i.prefix.as_deref().unwrap_or("/")).await {
            Ok(folders) => serde_json::to_string_pretty(&folders).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List artifacts by folder, session, owner, type, or class")]
    async fn list_artifacts(&self, Parameters(i): Parameters<ListArtifactsInput>) -> String {
        match self.metadata.list_artifacts(i.folder.as_deref(), i.session_id.as_deref(), i.artifact_class.as_ref()).await {
            Ok(arts) => {
                let metas: Vec<ArtifactMetadata> = arts.iter().map(ArtifactMetadata::from).collect();
                serde_json::to_string_pretty(&metas).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Inspect provenance, retention, access, hash, source")]
    async fn get_artifact_metadata(&self, Parameters(i): Parameters<GetArtifactMetadataInput>) -> String {
        match self.metadata.get_artifact(&i.artifact_id).await {
            Ok(art) => {
                let mut meta = ArtifactMetadata::from(&art);
                if let Ok(Some(v)) = self.metadata.get_current_version(&i.artifact_id).await {
                    meta.current_version = Some(VersionSummary {
                        version_id: v.version_id, version_number: v.version_number,
                        mime_type: v.mime_type, size_bytes: v.size_bytes,
                        sha256: v.sha256, created_at: v.created_at,
                    });
                }
                serde_json::to_string_pretty(&meta).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Read artifact content when policy permits")]
    async fn read_artifact(&self, Parameters(i): Parameters<ReadArtifactInput>) -> String {
        let art = match self.metadata.get_artifact(&i.artifact_id).await {
            Ok(a) => a,
            Err(e) => return format!("Error: {}", e),
        };
        if art.status == ArtifactStatus::PolicyGated {
            return "Access denied: artifact is policy-gated. Use request_artifact_access.".into();
        }
        let version = match self.metadata.get_current_version(&i.artifact_id).await {
            Ok(Some(v)) => v,
            Ok(None) => return "No content version available".into(),
            Err(e) => return format!("Error: {}", e),
        };
        match self.blobs.get(&version.storage_uri).await {
            Ok(data) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
                serde_json::to_string_pretty(&serde_json::json!({
                    "artifact_id": i.artifact_id,
                    "version_id": version.version_id,
                    "mime_type": version.mime_type,
                    "size_bytes": data.len(),
                    "content_base64": encoded,
                })).unwrap()
            }
            Err(e) => format!("Error reading blob: {}", e),
        }
    }

    #[tool(description = "Store a generated artifact with metadata and provenance")]
    async fn write_artifact(&self, Parameters(i): Parameters<WriteArtifactInput>) -> String {
        use base64::Engine;
        let data = match base64::engine::general_purpose::STANDARD.decode(&i.content_base64) {
            Ok(d) => d,
            Err(e) => return format!("Invalid base64: {}", e),
        };
        let artifact_id = format!("art_{}", Uuid::new_v4().simple());
        let version_id = format!("artv_{}", Uuid::new_v4().simple());
        let sha = format!("{:x}", Sha256::digest(&data));
        let uri = self.make_uri(&artifact_id, 1);

        if let Err(e) = self.blobs.put(&uri, &data, &i.mime_type).await {
            return format!("Storage error: {}", e);
        }

        let now = Utc::now();
        let art = Artifact {
            artifact_id: artifact_id.clone(),
            folder_path: i.folder_path,
            name: i.name,
            artifact_class: i.artifact_class.unwrap_or(ArtifactClass::SessionOutput),
            current_version_id: Some(version_id.clone()),
            owner: i.owner.unwrap_or_else(|| "system".into()),
            created_by_agent_id: None,
            session_id: i.session_id,
            data_class: None,
            risk_level: adk_mcp_sdk::risk::RiskLevel::Low,
            retention_class: i.retention_class.unwrap_or(RetentionClass::Standard),
            status: ArtifactStatus::Active,
            tags: i.tags.unwrap_or_default(),
            created_at: now, updated_at: now,
        };
        let ver = ArtifactVersion {
            version_id: version_id.clone(), artifact_id: artifact_id.clone(),
            version_number: 1, storage_uri: uri, mime_type: i.mime_type,
            size_bytes: data.len() as u64, sha256: sha,
            created_by: "system".into(), created_at: now,
        };

        let _ = self.metadata.put_artifact(art).await;
        let _ = self.metadata.put_version(ver).await;

        serde_json::to_string_pretty(&serde_json::json!({
            "artifact_id": artifact_id, "version_id": version_id, "status": "created"
        })).unwrap()
    }

    #[tool(description = "Add a new version to an existing artifact")]
    async fn create_artifact_version(&self, Parameters(i): Parameters<CreateVersionInput>) -> String {
        use base64::Engine;
        let data = match base64::engine::general_purpose::STANDARD.decode(&i.content_base64) {
            Ok(d) => d,
            Err(e) => return format!("Invalid base64: {}", e),
        };
        let mut art = match self.metadata.get_artifact(&i.artifact_id).await {
            Ok(a) => a,
            Err(e) => return format!("Error: {}", e),
        };
        let next_ver = self.metadata.get_current_version(&i.artifact_id).await
            .ok().flatten().map(|v| v.version_number + 1).unwrap_or(1);
        let version_id = format!("artv_{}", Uuid::new_v4().simple());
        let sha = format!("{:x}", Sha256::digest(&data));
        let uri = self.make_uri(&i.artifact_id, next_ver);

        if let Err(e) = self.blobs.put(&uri, &data, &i.mime_type).await {
            return format!("Storage error: {}", e);
        }

        let now = Utc::now();
        let ver = ArtifactVersion {
            version_id: version_id.clone(), artifact_id: i.artifact_id.clone(),
            version_number: next_ver, storage_uri: uri, mime_type: i.mime_type,
            size_bytes: data.len() as u64, sha256: sha,
            created_by: "system".into(), created_at: now,
        };
        art.current_version_id = Some(version_id.clone());
        art.updated_at = now;

        let _ = self.metadata.put_version(ver).await;
        let _ = self.metadata.update_artifact(art).await;

        serde_json::to_string_pretty(&serde_json::json!({
            "artifact_id": i.artifact_id, "version_id": version_id, "version_number": next_ver
        })).unwrap()
    }

    #[tool(description = "Create a redacted derived copy of a sensitive artifact")]
    async fn redact_artifact(&self, Parameters(i): Parameters<RedactArtifactInput>) -> String {
        use base64::Engine;
        let data = match base64::engine::general_purpose::STANDARD.decode(&i.redacted_content_base64) {
            Ok(d) => d,
            Err(e) => return format!("Invalid base64: {}", e),
        };
        let source = match self.metadata.get_artifact(&i.artifact_id).await {
            Ok(a) => a,
            Err(e) => return format!("Error: {}", e),
        };

        let new_id = format!("art_{}", Uuid::new_v4().simple());
        let version_id = format!("artv_{}", Uuid::new_v4().simple());
        let sha = format!("{:x}", Sha256::digest(&data));
        let uri = self.make_uri(&new_id, 1);
        let _ = self.blobs.put(&uri, &data, &i.mime_type).await;

        let now = Utc::now();
        let art = Artifact {
            artifact_id: new_id.clone(),
            folder_path: source.folder_path.clone(),
            name: format!("{} (redacted)", source.name),
            artifact_class: source.artifact_class.clone(),
            current_version_id: Some(version_id.clone()),
            owner: source.owner.clone(),
            created_by_agent_id: None, session_id: source.session_id.clone(),
            data_class: source.data_class.clone(),
            risk_level: adk_mcp_sdk::risk::RiskLevel::Low,
            retention_class: source.retention_class.clone(),
            status: ArtifactStatus::Active,
            tags: vec!["redacted".into()],
            created_at: now, updated_at: now,
        };
        let ver = ArtifactVersion {
            version_id: version_id.clone(), artifact_id: new_id.clone(),
            version_number: 1, storage_uri: uri, mime_type: i.mime_type,
            size_bytes: data.len() as u64, sha256: sha,
            created_by: "system".into(), created_at: now,
        };
        let edge = ArtifactEdge {
            edge_id: format!("edge_{}", Uuid::new_v4().simple()),
            from_artifact_id: i.artifact_id.clone(), to_artifact_id: new_id.clone(),
            edge_type: EdgeType::RedactedFrom, created_at: now,
        };

        let _ = self.metadata.put_artifact(art).await;
        let _ = self.metadata.put_version(ver).await;
        let _ = self.metadata.put_edge(edge).await;

        serde_json::to_string_pretty(&serde_json::json!({
            "redacted_artifact_id": new_id, "source_artifact_id": i.artifact_id
        })).unwrap()
    }

    #[tool(description = "Create an artifact derived from another with provenance link")]
    async fn derive_artifact(&self, Parameters(i): Parameters<DeriveArtifactInput>) -> String {
        use base64::Engine;
        let data = match base64::engine::general_purpose::STANDARD.decode(&i.content_base64) {
            Ok(d) => d,
            Err(e) => return format!("Invalid base64: {}", e),
        };
        let source = match self.metadata.get_artifact(&i.source_artifact_id).await {
            Ok(a) => a,
            Err(e) => return format!("Error: {}", e),
        };

        let new_id = format!("art_{}", Uuid::new_v4().simple());
        let version_id = format!("artv_{}", Uuid::new_v4().simple());
        let sha = format!("{:x}", Sha256::digest(&data));
        let uri = self.make_uri(&new_id, 1);
        let _ = self.blobs.put(&uri, &data, &i.mime_type).await;

        let now = Utc::now();
        let art = Artifact {
            artifact_id: new_id.clone(), folder_path: source.folder_path.clone(),
            name: i.name, artifact_class: source.artifact_class.clone(),
            current_version_id: Some(version_id.clone()), owner: source.owner.clone(),
            created_by_agent_id: None, session_id: source.session_id.clone(),
            data_class: source.data_class.clone(), risk_level: source.risk_level,
            retention_class: source.retention_class.clone(),
            status: ArtifactStatus::Active, tags: vec![],
            created_at: now, updated_at: now,
        };
        let ver = ArtifactVersion {
            version_id: version_id.clone(), artifact_id: new_id.clone(),
            version_number: 1, storage_uri: uri, mime_type: i.mime_type,
            size_bytes: data.len() as u64, sha256: sha,
            created_by: "system".into(), created_at: now,
        };
        let edge = ArtifactEdge {
            edge_id: format!("edge_{}", Uuid::new_v4().simple()),
            from_artifact_id: i.source_artifact_id.clone(), to_artifact_id: new_id.clone(),
            edge_type: i.edge_type.unwrap_or(EdgeType::DerivedFrom), created_at: now,
        };

        let _ = self.metadata.put_artifact(art).await;
        let _ = self.metadata.put_version(ver).await;
        let _ = self.metadata.put_edge(edge).await;

        serde_json::to_string_pretty(&serde_json::json!({
            "derived_artifact_id": new_id, "source_artifact_id": i.source_artifact_id
        })).unwrap()
    }

    #[tool(description = "Add a provenance edge between two artifacts")]
    async fn link_artifacts(&self, Parameters(i): Parameters<LinkArtifactsInput>) -> String {
        let edge = ArtifactEdge {
            edge_id: format!("edge_{}", Uuid::new_v4().simple()),
            from_artifact_id: i.from_artifact_id.clone(),
            to_artifact_id: i.to_artifact_id.clone(),
            edge_type: i.edge_type, created_at: Utc::now(),
        };
        match self.metadata.put_edge(edge).await {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({"status": "linked"})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Bundle related artifacts for audit or delivery")]
    async fn export_artifact_package(&self, Parameters(i): Parameters<ExportPackageInput>) -> String {
        let mut items = Vec::new();
        for id in &i.artifact_ids {
            if let Ok(art) = self.metadata.get_artifact(id).await {
                let mut meta = ArtifactMetadata::from(&art);
                if let Ok(Some(v)) = self.metadata.get_current_version(id).await {
                    meta.current_version = Some(VersionSummary {
                        version_id: v.version_id, version_number: v.version_number,
                        mime_type: v.mime_type, size_bytes: v.size_bytes,
                        sha256: v.sha256, created_at: v.created_at,
                    });
                }
                items.push(meta);
            }
        }
        serde_json::to_string_pretty(&serde_json::json!({
            "package_id": format!("pkg_{}", Uuid::new_v4().simple()),
            "artifact_count": items.len(), "artifacts": items,
        })).unwrap()
    }

    #[tool(description = "Validate hash and provenance trail")]
    async fn verify_artifact_integrity(&self, Parameters(i): Parameters<VerifyIntegrityInput>) -> String {
        let version = match self.metadata.get_current_version(&i.artifact_id).await {
            Ok(Some(v)) => v,
            Ok(None) => return "No version to verify".into(),
            Err(e) => return format!("Error: {}", e),
        };
        match self.blobs.get(&version.storage_uri).await {
            Ok(data) => {
                let actual_hash = format!("{:x}", Sha256::digest(&data));
                let valid = actual_hash == version.sha256;
                serde_json::to_string_pretty(&serde_json::json!({
                    "artifact_id": i.artifact_id,
                    "version_id": version.version_id,
                    "expected_sha256": version.sha256,
                    "actual_sha256": actual_hash,
                    "integrity_valid": valid,
                    "size_bytes": data.len(),
                })).unwrap()
            }
            Err(e) => format!("Error reading blob: {}", e),
        }
    }

    #[tool(description = "Trace provenance chain for an artifact")]
    async fn get_artifact_lineage(&self, Parameters(i): Parameters<GetLineageInput>) -> String {
        let from = self.metadata.get_edges_from(&i.artifact_id).await.unwrap_or_default();
        let to = self.metadata.get_edges_to(&i.artifact_id).await.unwrap_or_default();
        serde_json::to_string_pretty(&serde_json::json!({
            "artifact_id": i.artifact_id,
            "derived_to": from, "derived_from": to,
        })).unwrap()
    }

    #[tool(description = "Update retention policy on an artifact")]
    async fn set_retention_class(&self, Parameters(i): Parameters<SetRetentionInput>) -> String {
        match self.metadata.get_artifact(&i.artifact_id).await {
            Ok(mut art) => {
                art.retention_class = i.retention_class.clone();
                art.updated_at = Utc::now();
                let _ = self.metadata.update_artifact(art).await;
                serde_json::to_string_pretty(&serde_json::json!({
                    "artifact_id": i.artifact_id, "retention_class": i.retention_class
                })).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Request access to a policy-gated artifact")]
    async fn request_artifact_access(&self, Parameters(i): Parameters<RequestAccessInput>) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "artifact_id": i.artifact_id,
            "status": "access_requested",
            "reason": i.reason,
            "message": "Access request submitted for governance review"
        })).unwrap()
    }

    #[tool(description = "Delete artifact only if retention policy permits")]
    async fn delete_artifact_if_allowed(&self, Parameters(i): Parameters<DeleteArtifactInput>) -> String {
        let art = match self.metadata.get_artifact(&i.artifact_id).await {
            Ok(a) => a,
            Err(e) => return format!("Error: {}", e),
        };
        match art.retention_class {
            RetentionClass::Audit | RetentionClass::PaymentEvidence | RetentionClass::BuildRelease => {
                return format!("Deletion blocked: retention class {:?} prevents deletion", art.retention_class);
            }
            _ => {}
        }
        if let Some(ref vid) = art.current_version_id {
            if let Ok(v) = self.metadata.get_version(vid).await {
                let _ = self.blobs.delete(&v.storage_uri).await;
            }
        }
        let _ = self.metadata.delete_artifact(&i.artifact_id).await;
        serde_json::to_string_pretty(&serde_json::json!({
            "artifact_id": i.artifact_id, "status": "deleted"
        })).unwrap()
    }
}

#[async_trait::async_trait]
impl HealthCheck for ArtifactStoreServer {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            message: Some("operational".into()),
            latency_ms: Some(1),
        }
    }
}
