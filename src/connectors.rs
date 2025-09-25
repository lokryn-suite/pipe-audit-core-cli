use anyhow::Result;
use std::io::Read;

use crate::contracts::schema::Source;
use crate::profiles::Profiles;

/// Common interface for all connectors
#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    fn scheme(&self) -> &'static str;

    async fn list(&self, prefix: &str) -> Result<Vec<String>>;

    async fn fetch(&self, source: &str) -> Result<Box<dyn Read>>;
}

// bring in each connector implementation
pub mod azure;
pub mod gcs;
pub mod local;
pub mod s3;
pub mod sftp;

pub use azure::AzureConnector;
pub use gcs::GcsConnector;
pub use local::LocalConnector;
pub use s3::S3Connector;
pub use sftp::SftpConnector;

/// Factory: pick the right connector based on location type and profiles
pub async fn from_connection_string_with_profile(
    url: &str,
    source: &Source,
    profiles: &Profiles,
) -> Result<Box<dyn Connector>> {
    let profile = if let Some(profile_name) = &source.profile {
        profiles
            .get(profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile_name))?
    } else {
        return Err(anyhow::anyhow!("No profile specified for remote source"));
    };

    match source.r#type.as_str() {
        "s3" => {
            let parsed_url = url::Url::parse(url)?;
            Ok(Box::new(
                S3Connector::from_profile_and_url(profile, &parsed_url).await?,
            ))
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported connector type: {}",
            source.r#type
        )),
    }
}
