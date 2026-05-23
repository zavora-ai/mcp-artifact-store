use std::sync::Arc;
use mcp_artifact_store::{local::LocalBlobStore, memory::MemoryMetadataStore, server::ArtifactStoreServer};
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Artifact Store MCP server");

    let blob_root = std::env::var("ARTIFACT_STORE_PATH")
        .unwrap_or_else(|_| "./artifacts".into());

    let metadata = Arc::new(MemoryMetadataStore::new());
    let blobs: Arc<dyn mcp_artifact_store::BlobStore> = Arc::new(
        LocalBlobStore::new(blob_root.clone().into())
    );

    blobs.health_check().await?;
    tracing::info!(path = %blob_root, "Local blob store ready");

    let server = ArtifactStoreServer::new(metadata, blobs);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
