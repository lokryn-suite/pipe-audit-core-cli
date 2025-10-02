use anyhow::Result;
use std::io::Read;

/// Common interface for all connectors
#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    async fn fetch(&self, source: &str) -> Result<Box<dyn Read>>;
}

// bring in each connector implementation
pub mod azure;
pub mod fetch;
pub mod gcs;
pub mod local;
pub mod s3;
pub mod sftp;

pub use azure::AzureConnector;
pub use gcs::GCSConnector;
pub use local::LocalConnector;
pub use s3::S3Connector;
