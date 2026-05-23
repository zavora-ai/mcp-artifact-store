use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactClass {
    SessionOutput,
    AgentEvidence,
    GovernanceEvidence,
    PaymentEvidence,
    BuildOutput,
    TemplateAsset,
    MemoryExport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetentionClass {
    Ephemeral,
    Standard,
    Session,
    PiiRestricted,
    PaymentEvidence,
    Audit,
    BuildRelease,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Active,
    PolicyGated,
    Redacted,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DerivedFrom,
    RedactedFrom,
    BundledInto,
    ExportedFrom,
    GeneratedBy,
    ValidatedBy,
    ApprovedBy,
    EvidenceFor,
}

/// Core artifact record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: String,
    pub folder_path: String,
    pub name: String,
    pub artifact_class: ArtifactClass,
    pub current_version_id: Option<String>,
    pub owner: String,
    pub created_by_agent_id: Option<String>,
    pub session_id: Option<String>,
    pub data_class: Option<String>,
    pub risk_level: adk_mcp_sdk::risk::RiskLevel,
    pub retention_class: RetentionClass,
    pub status: ArtifactStatus,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Immutable version of an artifact's content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactVersion {
    pub version_id: String,
    pub artifact_id: String,
    pub version_number: u32,
    pub storage_uri: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

/// Provenance edge between artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEdge {
    pub edge_id: String,
    pub from_artifact_id: String,
    pub to_artifact_id: String,
    pub edge_type: EdgeType,
    pub created_at: DateTime<Utc>,
}

/// Metadata returned to agents (safe to expose).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub artifact_id: String,
    pub folder_path: String,
    pub name: String,
    pub artifact_class: ArtifactClass,
    pub owner: String,
    pub session_id: Option<String>,
    pub risk_level: adk_mcp_sdk::risk::RiskLevel,
    pub retention_class: RetentionClass,
    pub status: ArtifactStatus,
    pub current_version: Option<VersionSummary>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub version_id: String,
    pub version_number: u32,
    pub mime_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Artifact> for ArtifactMetadata {
    fn from(a: &Artifact) -> Self {
        Self {
            artifact_id: a.artifact_id.clone(),
            folder_path: a.folder_path.clone(),
            name: a.name.clone(),
            artifact_class: a.artifact_class.clone(),
            owner: a.owner.clone(),
            session_id: a.session_id.clone(),
            risk_level: a.risk_level,
            retention_class: a.retention_class.clone(),
            status: a.status.clone(),
            current_version: None,
            tags: a.tags.clone(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}
