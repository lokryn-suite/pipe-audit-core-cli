use super::Connector;
use crate::profiles::Profile;
use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use std::io::{Cursor, Read};

pub struct S3Connector {
    client: S3Client,
    bucket: String,
}

impl S3Connector {
    pub async fn from_profile_and_url(profile: &Profile, url: &url::Url) -> Result<Self> {
        // Expect s3://bucket/key style URLs
        let bucket = url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid S3 URL: missing bucket name"))?
            .to_string();

        let region = profile
            .region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());

        // Build base AWS config
        let mut config_loader =
            aws_config::defaults(BehaviorVersion::latest()).region(aws_config::Region::new(region));

        if let Some(endpoint) = &profile.endpoint {
            config_loader = config_loader.endpoint_url(endpoint);
        }

        let base_config = config_loader.load().await;

        // Build S3 config with optional path-style
        let mut s3_config = aws_sdk_s3::config::Builder::from(&base_config);
        if profile.path_style.unwrap_or(false) {
            s3_config = s3_config.force_path_style(true);
        }

        let client = S3Client::from_conf(s3_config.build());

        Ok(S3Connector { client, bucket })
    }

    fn parse_s3_path(&self, path: &str) -> Result<String> {
        if path.starts_with("s3://") {
            let url = url::Url::parse(path)?;
            Ok(url.path().trim_start_matches('/').to_string())
        } else {
            Ok(path.to_string())
        }
    }
}

#[async_trait::async_trait]
impl Connector for S3Connector {
    fn scheme(&self) -> &'static str {
        "s3"
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix_key = self.parse_s3_path(prefix)?;

        let resp = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix_key)
            .send()
            .await
            .context("Failed to list S3 objects")?;

        Ok(resp
            .contents
            .unwrap_or_default()
            .into_iter()
            .filter_map(|obj| obj.key)
            .map(|key| format!("s3://{}/{}", self.bucket, key))
            .collect())
    }

    async fn fetch(&self, location: &str) -> Result<Box<dyn Read>> {
        let key = self.parse_s3_path(location)?;

        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .context("Failed to fetch S3 object")?;

        let data = resp
            .body
            .collect()
            .await
            .context("Failed to read S3 object body")?
            .into_bytes();

        Ok(Box::new(Cursor::new(data)))
    }
}
