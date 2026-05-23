# Artifact Store MCP — Design Specification

## Decision

ADK-Rust Enterprise artifacts are a **structured artifact registry** backed by blob storage — not just "files in S3," and not a package registry.

> An **artifact** is a durable, policy-governed runtime output or input associated with an agent, session, workflow, skill, payment, build, or audit process, stored with metadata, provenance, access controls, lifecycle rules, and integrity verification.

## Architecture

```
Artifact Registry = metadata DB + object/blob store + provenance graph + policy layer
```

### Storage Layers

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Blob store | S3 / GCS / Azure Blob / MinIO | Content storage |
| Metadata store | Postgres (or DynamoDB serverless) | Queries, policy, ownership |
| Event/audit store | Append-only log | Access and lifecycle events |
| Provenance graph | Postgres edges table | Lineage and derivation |
| Search (optional) | OpenSearch | Full-text artifact search |

---

## Artifact Classes

| Class | Examples |
|-------|----------|
| Session output | Final answer, report, transcript, screenshot, generated JSON |
| Agent evidence | Tool result snapshot, source bundle, cited document set |
| Governance evidence | Policy decision log, approval record, audit pack |
| Payment evidence | Payment intent, receipt preview, ledger snapshot, refund evidence |
| Build output | Generated Rust project, Cargo.lock, SBOM, deploy bundle |
| Template asset | Fixture pack, skill lockfile, policy template |
| Memory export | Scoped recall snapshot, redacted profile export |

### Classification Hierarchy

```
ephemeral trace event       → not stored
temporary working file      → not stored (or short TTL)
session artifact            → stored, standard retention
governed artifact           → stored, policy-controlled
compliance artifact         → stored, long retention, immutable, signed
```

---

## Data Model

### artifacts

```yaml
artifact_id: art_44BD
tenant_id: tenant_123
workspace_id: ws_prod
environment: production
folder_path: /Production/sessions/ses_9F72
name: refund-intent.json
artifact_class: payment_evidence
logical_type: PaymentIntent
current_version_id: artv_001
owner_team: Customer Success
created_by_agent_id: realtime_support_voice
session_id: ses_9F72
data_class: financial
risk_level: medium
access_level: restricted
retention_class: payment_evidence
status: active | policy_gated | redacted | archived | deleted
created_at: timestamp
updated_at: timestamp
```

### artifact_versions

```yaml
version_id: artv_001
artifact_id: art_44BD
version_number: 1
storage_uri: s3://adk-prod-artifacts/tenant/ws/art_44BD/v1/blob
mime_type: application/json
size_bytes: 4096
sha256: <hash>
metadata_sha256: <hash>
signature: <optional signed hash>
immutable: true
created_by: agent/runtime/user
created_at: timestamp
```

### artifact_edges (provenance graph)

```yaml
edge_id: edge_001
from_artifact_id: art_raw_transcript
to_artifact_id: art_redacted_transcript
edge_type: derived_from | redacted_from | bundled_into | exported_from | generated_by | validated_by | approved_by | evidence_for
policy_id: pol_memory_privacy
created_at: timestamp
```

### Additional tables

- `artifact_access_grants` — per-artifact ACLs
- `artifact_policy_labels` — data class, risk, governance tags
- `artifact_events` — append-only audit log
- `artifact_retention_jobs` — lifecycle worker state
- `artifact_redactions` — redaction metadata
- `artifact_collections` — grouping for export packages

---

## Mutability Rules

Artifacts are **content-immutable by default**. Updates create new versions.

| Artifact type | Mutability |
|---------------|-----------|
| Audit pack | Immutable |
| Payment evidence | Immutable |
| Policy decision log | Immutable / append-only |
| Report draft | Versioned |
| Generated code bundle | Immutable per build |
| Transcript | Raw immutable, redacted copy versioned |
| Spreadsheet forecast | Versioned |
| Temporary working file | Mutable only before promotion |

---

## Retention Model

Policy-driven, not user-decided.

| Retention class | Example | Default |
|----------------|---------|---------|
| ephemeral | Scratch files, intermediate tool outputs | Hours/days |
| standard | Reports, summaries, JSON outputs | 90 days |
| session | Session artifacts | 30–180 days |
| pii_restricted | Transcripts, HR, customer data | Short + redaction |
| payment_evidence | Receipts, intents, ledger snapshots | Long legal retention |
| audit | Approval records, policy decisions | 1–7 years |
| build_release | Deploy bundles, SBOMs, rollback packages | Tied to release policy |

Deletion handled by lifecycle worker consulting Governance Policy MCP.

---

## Access Control

Layered:

```
workspace / tenant → environment → folder → artifact → version → data class → purpose → session ownership → policy decision
```

`read_artifact` should return one of:
- Artifact content
- Signed short-lived download URL
- Redacted version
- Metadata only
- Policy denial
- Approval required

The Artifact MCP runs with service identity (via Credentials Vault), validates policy, then serves content. Agents never get raw storage credentials.

---

## Redaction Model

Redaction produces a **new derived artifact**, never mutates the original.

```
art_raw_transcript_v1 → art_redacted_transcript_v1
```

Redacted artifact stores:
- `source_artifact_id`
- `redaction_policy_id`
- `redaction_method`
- `fields_removed`
- `created_by`, `created_at`, `hash`

---

## Provenance

Minimum provenance fields per artifact:

- `artifact_id`, `version_id`
- `created_at`, `created_by_agent_id`, `created_by_agent_version`
- `session_id`, `workflow_id`, `skill_id`, `skill_version`
- `environment`, `model_provider`, `model_name`
- `tool_calls_used`
- `source_artifact_ids`, `source_resource_ids`
- `policy_decision_ids`, `approval_ids`, `payment_intent_ids`
- `hash_sha256`, `signature`

Edge types for lineage:
- `derived_from`, `redacted_from`, `bundled_into`, `exported_from`
- `generated_by`, `validated_by`, `approved_by`, `evidence_for`

---

## Integrity

Every artifact version has:
- SHA-256 hash of content
- Content length + MIME type
- Storage URI
- Metadata hash
- Optional signature (for high-risk: payment, audit, build)

Signature: `Sign(platform_key, artifact_id + version_id + hash + metadata_hash)`

---

## MCP Tool Surface

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_folders` | Browse workspace folder hierarchy | Read-only |
| `list_artifacts` | List artifacts by folder, session, owner, type, class | Read-only |
| `get_artifact_metadata` | Inspect provenance, retention, hash, source | Read-only |
| `read_artifact` | Read content (policy-gated) | Read-only (gated) |
| `write_artifact` | Store a generated artifact | Internal write |
| `create_artifact_version` | Add new version to existing artifact | Internal write |
| `redact_artifact` | Create redacted derived copy | Internal write |
| `derive_artifact` | Create artifact derived from another | Internal write |
| `link_artifacts` | Add provenance edge between artifacts | Internal write |
| `export_artifact_package` | Bundle related artifacts for audit/delivery | Read-only |
| `verify_artifact_integrity` | Validate hash and provenance trail | Read-only |
| `get_artifact_lineage` | Trace provenance chain | Read-only |
| `set_retention_class` | Update retention policy on artifact | Internal write |
| `request_artifact_access` | Request access when policy-gated | External write |
| `delete_artifact_if_allowed` | Delete only if retention policy permits | Internal write |

---

## Integration Points

| Server | Relationship |
|--------|-------------|
| Credentials Vault MCP | Artifact MCP uses vault for storage credentials |
| Governance Policy MCP | `write_artifact`, `read_artifact`, `delete` call policy evaluation |
| Session Memory MCP | References artifacts by ID, stores summaries not content |
| ADK-Payments MCP | Payment evidence artifacts: immutable, signed, long-retained |
| Visual Builder MCP | Build outputs stored as artifacts |
| MCP Registry | Artifact MCP registered with tool allow-lists |

---

## Backend Trait

```rust
#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn health_check(&self) -> Result<(), ArtifactError>;
    async fn put_blob(&self, uri: &str, data: &[u8], mime: &str) -> Result<(), ArtifactError>;
    async fn get_blob(&self, uri: &str) -> Result<Vec<u8>, ArtifactError>;
    async fn delete_blob(&self, uri: &str) -> Result<(), ArtifactError>;
    async fn generate_signed_url(&self, uri: &str, ttl: Duration) -> Result<String, ArtifactError>;
}
```

Metadata operations go through the metadata store (Postgres/DynamoDB), not the blob backend.

---

## Implementation Plan

1. Define core types (`Artifact`, `ArtifactVersion`, `ArtifactEdge`, `RetentionClass`, etc.)
2. Implement metadata store trait (start with in-memory, then Postgres)
3. Implement blob store trait (S3, GCS, local filesystem)
4. Implement 15 MCP tools
5. Wire up Credentials Vault for storage auth
6. Add integrity verification (SHA-256 + optional signing)
7. Add retention lifecycle worker
8. Tests with mock stores + real S3/GCS
