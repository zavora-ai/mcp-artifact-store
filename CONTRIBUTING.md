# Contributing to Artifact Store MCP

Thank you for your interest in contributing! This server is part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) ecosystem.

## Getting Started

```bash
# Requires Rust 1.85+ (2024 edition)
git clone https://github.com/zavora-ai/mcp-artifact-store
cd mcp-artifact-store
cargo build --features all-backends
cargo test
```

## Development Workflow

- `feature/description` — New features
- `fix/description` — Bug fixes
- `backend/name` — New blob storage backends

## Adding a New Blob Backend

1. Create `src/my_backend.rs`
2. Implement the `BlobStore` trait
3. Add a feature flag in `Cargo.toml`
4. Add the module to `src/lib.rs` behind `#[cfg(feature = "my-backend")]`

## Code Standards

- `cargo clippy --features all-backends`
- `cargo fmt`
- All artifacts must be content-immutable (new versions, not overwrites)
- All writes must compute SHA-256 hash

## Pull Requests

- Keep PRs focused on a single change
- Include tests for new functionality
- Update CHANGELOG.md
