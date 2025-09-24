use super::Connector;
use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use std::io::{Read, Cursor};

pub struct S3Connector {
    client: S3Client,
    bucket: String,
    region: Option<String>,
}

impl S3Connector {
    pub async fn from_url_async(url: &url::Url) -> Result<Self> {
        let bucket = url.host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid S3 URL: missing bucket name"))?
            .to_string();

        let query_params: std::collections::HashMap<String, String> = url.query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let region = query_params.get("region")
            .cloned()
            .unwrap_or_else(|| "us-east-1".to_string());

        // Load AWS configuration
        let mut config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()));

        // Check for custom endpoint (for MinIO compatibility)
        if let Some(endpoint) = query_params.get("endpoint") {
            config_builder = config_builder.endpoint_url(endpoint);
        }

        // Check for force path style (required for MinIO)
        if query_params.get("force_path_style").map(|v| v == "true").unwrap_or(false) {
            // This will be handled in the client configuration
        }

        let config = config_builder.load().await;
        let mut client_config = aws_sdk_s3::config::Builder::from(&config);
        
        // Enable path-style addressing for MinIO
        if query_params.get("force_path_style").map(|v| v == "true").unwrap_or(false) {
            client_config = client_config.force_path_style(true);
        }

        let client = S3Client::from_conf(client_config.build());

        Ok(S3Connector {
            client,
            bucket,
            region: Some(region),
        })
    }

    fn parse_s3_path(&self, path: &str) -> Result<String> {
        // Handle both full s3:// URLs and just key paths
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
        
        let resp = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix_key)
            .send()
            .await
            .context("Failed to list S3 objects")?;

        let mut files = Vec::new();
        if let Some(objects) = resp.contents {
            for object in objects {
                if let Some(key) = object.key {
                    files.push(format!("s3://{}/{}", self.bucket, key));
                }
            }
        }

        Ok(files)
    }

    async fn fetch(&self, location: &str) -> Result<Box<dyn Read>> {
        let key = self.parse_s3_path(location)?;
        
        let resp = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .context("Failed to fetch S3 object")?;

        // Read the entire object into memory as bytes
        let data = resp.body
            .collect()
            .await
            .context("Failed to read S3 object body")?
            .into_bytes();

        // Return a Cursor over the bytes, which implements Read
        Ok(Box::new(Cursor::new(data)))
    }
}
