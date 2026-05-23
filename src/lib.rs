//! # Artifact Store MCP Server
//!
//! Governed artifact registry for ADK-Rust Enterprise — content-immutable,
//! versioned, provenance-linked, hash-verified artifacts with policy-driven lifecycle.

pub mod error;
pub mod store;
pub mod types;
pub mod memory;
pub mod server;

#[cfg(feature = "local")]
pub mod local;

#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "gcs")]
pub mod gcs;

pub use error::StoreError;
pub use store::{BlobStore, MetadataStore};
pub use types::*;
