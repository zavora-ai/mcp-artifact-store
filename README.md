# Artifact Store MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-artifact-store.svg)](https://crates.io/crates/mcp-artifact-store)
[![Docs.rs](https://docs.rs/mcp-artifact-store/badge.svg)](https://docs.rs/mcp-artifact-store)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Governed artifact registry for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Content-immutable, versioned, provenance-linked, hash-verified artifacts with policy-driven lifecycle.

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-artifact-store/main/docs/architecture.svg" alt="Artifact Store MCP Architecture" width="800"/>
</p>

## Key Principles

- **Content-immutable** — updates create new versions, never overwrite
- **Provenance-linked** — every artifact tracks its lineage
- **Hash-verified** — SHA-256 integrity on every version
- **Policy-driven lifecycle** — retention classes prevent premature deletion
- **Redaction by derivation** — redacted copies are new artifacts, originals preserved

## Tools (15)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `write_artifact` | Store a generated artifact | Internal write |
| `create_artifact_version` | Add new version to existing artifact | Internal write |
| `read_artifact` | Read content (policy-gated) | Read-only |
| `list_artifacts` | List artifacts by folder, session, class | Read-only |
| `list_folders` | Browse workspace folder hierarchy | Read-only |
| `get_artifact_metadata` | Inspect provenance, retention, hash | Read-only |
| `verify_artifact_integrity` | Validate SHA-256 hash | Read-only |
| `get_artifact_lineage` | Trace provenance chain | Read-only |
| `export_artifact_package` | Bundle artifacts for audit/delivery | Read-only |
| `derive_artifact` | Create artifact derived from another | Internal write |
| `redact_artifact` | Create redacted derived copy | Internal write |
| `link_artifacts` | Add provenance edge | Internal write |
| `set_retention_class` | Update retention policy | Internal write |
| `request_artifact_access` | Request access to gated artifact | External write |
| `delete_artifact_if_allowed` | Delete only if retention permits | Internal write |

## Example Prompts & Outputs

### Store a report

**Prompt:** "Save this analysis report as a session artifact"

**Tool call:** `write_artifact`
```json
{
  "name": "quarterly-analysis.json",
  "folder_path": "/reports/2026-Q2",
  "content_base64": "eyJyZXZlbnVlIjogIjEuMk0iLCAiZ3Jvd3RoIjogIjE1JSJ9",
  "mime_type": "application/json",
  "artifact_class": "session_output",
  "retention_class": "standard",
  "tags": ["finance", "Q2-2026"]
}
```

**Output:**
```json
{
  "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904",
  "version_id": "artv_b9c87a30425843ff8837672beee05b37",
  "status": "created"
}
```

---

### Inspect an artifact

**Prompt:** "Show me the metadata for that report"

**Tool call:** `get_artifact_metadata`
```json
{ "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904" }
```

**Output:**
```json
{
  "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904",
  "folder_path": "/reports/2026-Q2",
  "name": "quarterly-analysis.json",
  "artifact_class": "session_output",
  "owner": "system",
  "session_id": "ses_test_001",
  "risk_level": "low",
  "retention_class": "standard",
  "status": "active",
  "current_version": {
    "version_id": "artv_b9c87a30425843ff8837672beee05b37",
    "version_number": 1,
    "mime_type": "application/json",
    "size_bytes": 42,
    "sha256": "da09a1f13a3d728bccc87bc001a2950163c8a1cd96161a0390da1660467ffb86",
    "created_at": "2026-05-23T11:29:42Z"
  },
  "tags": ["finance", "Q2-2026"],
  "created_at": "2026-05-23T11:29:42Z",
  "updated_at": "2026-05-23T11:29:42Z"
}
```

---

### Verify integrity

**Prompt:** "Verify the integrity of this artifact"

**Tool call:** `verify_artifact_integrity`
```json
{ "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904" }
```

**Output:**
```json
{
  "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904",
  "version_id": "artv_b9c87a30425843ff8837672beee05b37",
  "expected_sha256": "da09a1f13a3d728bccc87bc001a2950163c8a1cd96161a0390da1660467ffb86",
  "actual_sha256": "da09a1f13a3d728bccc87bc001a2950163c8a1cd96161a0390da1660467ffb86",
  "integrity_valid": true,
  "size_bytes": 42
}
```

---

### Derive a summary from a source artifact

**Prompt:** "Create an executive summary derived from the quarterly report"

**Tool call:** `derive_artifact`
```json
{
  "source_artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904",
  "name": "executive-summary.md",
  "content_base64": "IyBRMiBTdW1tYXJ5CgpSZXZlbnVlIGdyZXcgMTUlIHRvICQxLjJNLg==",
  "mime_type": "text/markdown"
}
```

**Output:**
```json
{
  "derived_artifact_id": "art_aa3a2a0a5c3744988393209258b32a15",
  "source_artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904"
}
```

---

### Trace provenance

**Prompt:** "Show me the lineage of the quarterly report"

**Tool call:** `get_artifact_lineage`
```json
{ "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904" }
```

**Output:**
```json
{
  "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904",
  "derived_from": [],
  "derived_to": [
    {
      "edge_id": "edge_98637dc4d55c40aa9c03e238551230f9",
      "edge_type": "derived_from",
      "from_artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904",
      "to_artifact_id": "art_aa3a2a0a5c3744988393209258b32a15",
      "created_at": "2026-05-23T11:30:26Z"
    }
  ]
}
```

---

### Retention enforcement

**Prompt:** "Mark this as an audit artifact and try to delete it"

**Tool call:** `set_retention_class`
```json
{ "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904", "retention_class": "audit" }
```

**Output:**
```json
{ "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904", "retention_class": "audit" }
```

**Tool call:** `delete_artifact_if_allowed`
```json
{ "artifact_id": "art_c2a2bcf2f86f48a88eba7168612e2904" }
```

**Output:**
```
Deletion blocked: retention class Audit prevents deletion
```

---

### Redact sensitive content

**Prompt:** "Create a redacted version of this transcript for sharing"

**Tool call:** `redact_artifact`
```json
{
  "artifact_id": "art_transcript_001",
  "redacted_content_base64": "VHJhbnNjcmlwdDogW1JFREFDVEVEXSBkaXNjdXNzZWQgcHJpY2luZy4=",
  "mime_type": "text/plain",
  "fields_removed": ["customer_name", "phone_number", "account_id"]
}
```

**Output:**
```json
{
  "redacted_artifact_id": "art_f7b2c1d4e5a6...",
  "source_artifact_id": "art_transcript_001"
}
```

---

### Export audit package

**Prompt:** "Bundle these artifacts for the compliance audit"

**Tool call:** `export_artifact_package`
```json
{
  "artifact_ids": [
    "art_c2a2bcf2f86f48a88eba7168612e2904",
    "art_aa3a2a0a5c3744988393209258b32a15"
  ]
}
```

**Output:**
```json
{
  "package_id": "pkg_4f8a2b1c...",
  "artifact_count": 2,
  "artifacts": [
    { "artifact_id": "art_c2a2...", "name": "quarterly-analysis.json", ... },
    { "artifact_id": "art_aa3a...", "name": "executive-summary.md", ... }
  ]
}
```

---

## Backends

| Layer | Backend | Feature Flag |
|-------|---------|-------------|
| Blob storage | Local filesystem | `local` (default) |
| Blob storage | AWS S3 | `s3` |
| Blob storage | Google Cloud Storage | `gcs` |
| Metadata | In-memory (dev/testing) | Always available |

## Installation

### Build from source

```bash
git clone https://github.com/zavora-ai/mcp-artifact-store
cd mcp-artifact-store
cargo build --release --features all-backends
```

### Claude Desktop

```json
{
  "mcpServers": {
    "artifact-store": {
      "command": "/path/to/mcp-artifact-store",
      "env": { "ARTIFACT_STORE_PATH": "~/artifacts" }
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "artifact-store": {
      "command": "/path/to/mcp-artifact-store",
      "env": { "ARTIFACT_STORE_PATH": "~/artifacts" }
    }
  }
}
```

### Codex / Cursor / Windsurf / Antigravity / Open Code

Same pattern — point `command` to the binary and set `ARTIFACT_STORE_PATH`.

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ARTIFACT_STORE_PATH` | Local blob storage root | `./artifacts` |
| `AWS_REGION` | S3 region (s3 feature) | — |
| `ARTIFACT_S3_BUCKET` | S3 bucket name | — |
| `GCP_PROJECT_ID` | GCP project (gcs feature) | — |
| `ARTIFACT_GCS_BUCKET` | GCS bucket name | — |

## Artifact Classes

| Class | Examples |
|-------|----------|
| `session_output` | Reports, transcripts, screenshots, JSON |
| `agent_evidence` | Tool result snapshots, cited documents |
| `governance_evidence` | Policy decisions, approval records, audit packs |
| `payment_evidence` | Receipts, intents, ledger snapshots |
| `build_output` | Generated code, SBOMs, deploy bundles |
| `template_asset` | Fixtures, skill lockfiles, workflow templates |
| `memory_export` | Recall snapshots, redacted profile exports |

## Retention Classes

| Class | Default Retention | Deletable? |
|-------|-------------------|------------|
| `ephemeral` | Hours/days | ✅ Yes |
| `standard` | 90 days | ✅ Yes |
| `session` | 30–180 days | ✅ Yes |
| `pii_restricted` | Short + redaction | ⚠️ Gated |
| `payment_evidence` | Legal retention | ❌ No |
| `audit` | 1–7 years | ❌ No |
| `build_release` | Tied to release | ❌ No |

## Provenance Edge Types

| Edge | Meaning |
|------|---------|
| `derived_from` | Artifact B was created from artifact A |
| `redacted_from` | Artifact B is a redacted copy of A |
| `bundled_into` | Artifact A was included in package B |
| `exported_from` | Artifact B was exported from system A |
| `generated_by` | Artifact was generated by a tool/agent |
| `validated_by` | Artifact was validated by a policy check |
| `approved_by` | Artifact was approved by a human/gate |
| `evidence_for` | Artifact serves as evidence for a decision |

## Documentation

| Document | Description |
|----------|-------------|
| [SPEC.md](SPEC.md) | Full design specification |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development guidelines |
| [SECURITY.md](SECURITY.md) | Vulnerability reporting |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
