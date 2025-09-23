use std::io::Read;
use anyhow::Result;
use url::Url;

/// Common interface for all connectors
pub trait Connector: Send + Sync {
    fn scheme(&self) -> &'static str;

    fn list(&self, prefix: &str) -> Result<Vec<String>>;

    fn fetch(&self, location: &str) -> Result<Box<dyn Read>>;
}

// bring in each connector implementation
pub mod local;
pub mod s3;
pub mod azure;
pub mod gcs;
pub mod sftp;

pub use local::LocalConnector;
pub use s3::S3Connector;
pub use azure::AzureConnector;
pub use gcs::GcsConnector;
pub use sftp::SftpConnector;

/// Factory: pick the right connector based on URI scheme
pub fn from_connection_string(conn: &str) -> Result<Box<dyn Connector>> {
    let url = Url::parse(conn)?;
    match url.scheme() {
        "file" => Ok(Box::new(LocalConnector::new())),
        "s3"   => Ok(Box::new(S3Connector::from_url(&url)?)),
        "azure" => Ok(Box::new(AzureConnector::from_url(&url)?)),
        "gcs"   => Ok(Box::new(GcsConnector::from_url(&url)?)),
        "sftp"  => Ok(Box::new(SftpConnector::from_url(&url)?)),
        _ => Err(anyhow::anyhow!("Unsupported scheme: {}", url.scheme())),
    }
}
