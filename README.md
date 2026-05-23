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
- **Provenance-linked** — every artifact tracks its lineage (derived_from, redacted_from, etc.)
- **Hash-verified** — SHA-256 integrity on every version
- **Policy-driven lifecycle** — retention classes prevent premature deletion
- **Redaction by derivation** — redacted copies are new artifacts, originals preserved

## Tools

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_folders` | Browse workspace folder hierarchy | Read-only |
| `list_artifacts` | List artifacts by folder, session, owner, class | Read-only |
| `get_artifact_metadata` | Inspect provenance, retention, hash, source | Read-only |
| `read_artifact` | Read content (policy-gated) | Read-only |
| `write_artifact` | Store a generated artifact | Internal write |
| `create_artifact_version` | Add new version to existing artifact | Internal write |
| `redact_artifact` | Create redacted derived copy | Internal write |
| `derive_artifact` | Create artifact derived from another | Internal write |
| `link_artifacts` | Add provenance edge between artifacts | Internal write |
| `export_artifact_package` | Bundle related artifacts for audit/delivery | Read-only |
| `verify_artifact_integrity` | Validate SHA-256 hash and provenance | Read-only |
| `get_artifact_lineage` | Trace provenance chain | Read-only |
| `set_retention_class` | Update retention policy | Internal write |
| `request_artifact_access` | Request access to policy-gated artifact | External write |
| `delete_artifact_if_allowed` | Delete only if retention permits | Internal write |

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

### Codex / Cursor / Windsurf / Open Code

Same pattern — point `command` to the binary and set `ARTIFACT_STORE_PATH`.

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ARTIFACT_STORE_PATH` | Local blob storage root | `./artifacts` |
| `AWS_REGION` | S3 region (when using s3 feature) | — |
| `ARTIFACT_S3_BUCKET` | S3 bucket name | — |
| `GCP_PROJECT_ID` | GCP project (when using gcs feature) | — |
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
| `ephemeral` | Hours/days | Yes |
| `standard` | 90 days | Yes |
| `session` | 30–180 days | Yes |
| `pii_restricted` | Short + redaction | Gated |
| `payment_evidence` | Legal retention | No |
| `audit` | 1–7 years | No |
| `build_release` | Tied to release | No |

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
