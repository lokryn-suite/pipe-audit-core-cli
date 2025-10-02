use crate::connectors::{AzureConnector, Connector, GCSConnector, LocalConnector, S3Connector};
use crate::contracts::schema::Source;
use crate::logging::error::{ValidationError, ValidationResult};
use crate::profiles::Profiles;
use std::io::Read;
use url::Url;

pub async fn fetch_data_from_source(
    source: &Source,
    profiles: &Profiles,
) -> ValidationResult<Vec<u8>> {
    let location = source
        .location
        .as_ref()
        .ok_or_else(|| ValidationError::Other("Source missing location".to_string()))?;

    match source.r#type.as_str() {
        "local" => {
            let connector = LocalConnector::new();
            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut buf = Vec::new();
            reader
                .read_to_end(&mut buf)
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            Ok(buf)
        }
        "s3" => {
            let profile_name = source
                .profile
                .as_ref()
                .ok_or_else(|| ValidationError::Other("S3 source requires profile".to_string()))?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| ValidationError::ProfileNotFound(profile_name.clone()))?;
            let url = Url::parse(location)
                .map_err(|_| ValidationError::Other("Invalid URL".to_string()))?;
            let connector = S3Connector::from_profile_and_url(profile, &url)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut buffer = Vec::new();
            reader
                .read_to_end(&mut buffer)
                .map_err(ValidationError::Io)?;
            Ok(buffer)
        }
        "azure" => {
            let profile_name = source.profile.as_ref().ok_or_else(|| {
                ValidationError::Other("Azure source requires profile".to_string())
            })?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| ValidationError::ProfileNotFound(profile_name.clone()))?;
            let url = Url::parse(location)
                .map_err(|_| ValidationError::Other("Invalid URL".to_string()))?;
            let connector = AzureConnector::from_profile_and_url(profile, &url)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut buffer = Vec::new();
            reader
                .read_to_end(&mut buffer)
                .map_err(ValidationError::Io)?;
            Ok(buffer)
        }
        "gcs" => {
            let profile_name = source
                .profile
                .as_ref()
                .ok_or_else(|| ValidationError::Other("GCS source requires profile".to_string()))?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| ValidationError::ProfileNotFound(profile_name.clone()))?;
            let url = Url::parse(location)
                .map_err(|_| ValidationError::Other("Invalid URL".to_string()))?;
            let connector = GCSConnector::from_profile_and_url(profile, &url)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| ValidationError::Connector(e.to_string()))?;
            let mut buffer = Vec::new();
            reader
                .read_to_end(&mut buffer)
                .map_err(ValidationError::Io)?;
            Ok(buffer)
        }
        _ => Err(ValidationError::Other(format!(
            "Unsupported source type: {}",
            source.r#type
        ))),
    }
}
