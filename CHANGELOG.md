# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-05-23

### Added

- **15 MCP tools** — full artifact lifecycle: write, version, read, redact, derive, link, export, verify, lineage, retention, access, delete
- **Content-immutable versioning** — updates create new versions, never overwrite blobs
- **Provenance graph** — edges between artifacts (derived_from, redacted_from, bundled_into, etc.)
- **SHA-256 integrity verification** — every version hashed, verifiable via `verify_artifact_integrity`
- **Retention enforcement** — audit, payment_evidence, and build_release artifacts cannot be deleted
- **Redaction by derivation** — `redact_artifact` creates a new artifact linked to the source
- **7 artifact classes** — session_output, agent_evidence, governance_evidence, payment_evidence, build_output, template_asset, memory_export
- **7 retention classes** — ephemeral, standard, session, pii_restricted, payment_evidence, audit, build_release
- **3 blob backends** — local filesystem, AWS S3, Google Cloud Storage
- **In-memory metadata store** — for development and testing
- **Policy-gated reads** — `read_artifact` respects artifact status
- **ADK-Rust Enterprise SDK integration** — `mcp-server.toml` manifest for registry onboarding
- **rmcp 1.7** — latest MCP protocol SDK
