//! Amazon S3 connector.
//!
//! Provides read/write access to S3 buckets using the AWS SDK for Rust.
//! Implements the generic `Connector` trait so it can be used
//! interchangeably with other backends (Azure, GCS, local).
//!
//! ## Responsibilities
//! - Construct an `S3Client` from a `Profile` and S3 URL.
//! - Support both virtual-hosted and path-style addressing.
//! - Upload (`put_object_from_url`) and fetch (`fetch`) objects.
//! - List objects under a given prefix.
//!
//! ## Expected URL format
//! - `s3://bucket/key`
//!
//! ## Profile fields used
//! - `region` (default: `us-east-1`)
//! - `endpoint` (optional, for custom endpoints / MinIO)
//! - `access_key` / `secret_key` (optional, overrides default credentials)
//! - `path_style` (optional, forces path-style addressing)

use super::Connector;
use crate::profiles::Profile;
use anyhow::{Context, Result, anyhow};
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Credentials;
use std::io::{Cursor, Read};

/// Concrete connector for S3.
pub struct S3Connector {
    client: S3Client,
    bucket: String,
}

impl S3Connector {
    /// Build an `S3Connector` from a profile and URL.
    ///
    /// - Extracts bucket name from the URL.
    /// - Loads AWS config (region, endpoint, credentials).
    /// - Applies path-style if requested.
    pub async fn from_profile_and_url(profile: &Profile, url: &url::Url) -> Result<Self> {
        // Expect s3://bucket/key style URLs
        let bucket = url
            .host_str()
            .ok_or_else(|| anyhow!("Invalid S3 URL: missing bucket name"))?
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
        let mut s3_config = aws_sdk_s3::config::Builder::from(&base_config);

        // Override credentials if explicitly provided in profile
        if let (Some(access_key), Some(secret_key)) = (&profile.access_key, &profile.secret_key) {
            if !access_key.is_empty() && !secret_key.is_empty() {
                let creds = Credentials::new(
                    access_key.clone(),
                    secret_key.clone(),
                    None,
                    None,
                    "profile",
                );
                s3_config = s3_config.credentials_provider(creds);
            }
        }

        // Force path-style if requested
        if profile.path_style.unwrap_or(false) {
            s3_config = s3_config.force_path_style(true);
        }

        let client = S3Client::from_conf(s3_config.build());

        Ok(S3Connector { client, bucket })
    }

    /// Upload an object to S3 given a full `s3://bucket/key` URL.
    pub async fn put_object_from_url(&self, s3_url: &str, data: &[u8]) -> Result<()> {
        let url = url::Url::parse(s3_url)?;
        let bucket = url
            .host_str()
            .ok_or_else(|| anyhow!("Invalid S3 URL: missing bucket"))?;
        let key = url.path().trim_start_matches('/');

        use aws_sdk_s3::primitives::ByteStream;

        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .context("Failed to upload S3 object")?;

        Ok(())
    }

    /// Normalize an S3 path into a key (strip `s3://bucket/` if present).
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
    /// Fetch an object from S3 and return it as a `Read` stream.
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
