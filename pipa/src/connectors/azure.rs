//! Azure Blob Storage connector.
//!
//! Provides read/write access to Azure Blob Storage using the REST API
//! and SharedKey authentication. Implements the generic `Connector`
//! trait so it can be used interchangeably with other backends (S3, GCS, local).
//!
//! ## Responsibilities
//! - Parse `connection_string` from a profile.
//! - Generate SharedKey authorization headers.
//! - Upload (`put_object_from_url`) and fetch (`fetch`) blobs.
//!
//! ## Profile fields used
//! - `connection_string` (must contain `AccountName` and `AccountKey`).
//!
//! ## Expected URL format
//! - `https://<account>.blob.core.windows.net/<container>/<blob>`

use crate::connectors::Connector;
use crate::profiles::Profile;
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::io::Read;
use url::Url;

/// Concrete connector for Azure Blob Storage.
pub struct AzureConnector {
    account_name: String,
    account_key: String,
    client: reqwest::Client,
}

impl AzureConnector {
    /// Build an `AzureConnector` from a profile and URL.
    ///
    /// Expects `connection_string` in the profile.
    pub async fn from_profile_and_url(profile: &Profile, _url: &Url) -> Result<Self> {
        let connection_string = profile
            .connection_string
            .as_ref()
            .ok_or_else(|| anyhow!("Azure profile missing connection_string"))?;

        let (account_name, account_key) = Self::parse_connection_string(connection_string)?;

        Ok(AzureConnector {
            account_name,
            account_key,
            client: reqwest::Client::new(),
        })
    }

    /// Parse `AccountName` and `AccountKey` from a connection string.
    fn parse_connection_string(connection_string: &str) -> Result<(String, String)> {
        let mut account_name = None;
        let mut account_key = None;

        for part in connection_string.split(';') {
            if let Some(name) = part.strip_prefix("AccountName=") {
                account_name = Some(name.to_string());
            } else if let Some(key) = part.strip_prefix("AccountKey=") {
                account_key = Some(key.to_string());
            }
        }

        match (account_name, account_key) {
            (Some(name), Some(key)) => Ok((name, key)),
            _ => bail!("Invalid connection string format"),
        }
    }

    /// Create a SharedKey authorization header for Azure Blob Storage.
    ///
    /// Builds the canonical string to sign and computes the HMAC-SHA256 signature.
    fn create_auth_header(
        &self,
        method: &str,
        url: &str,
        content_length: usize,
    ) -> Result<(String, String)> {
        let parsed_url = Url::parse(url)?;
        let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let resource = format!("/{}{}", self.account_name, parsed_url.path());

        // Canonical string differs for GET vs PUT
        let string_to_sign = if method == "GET" {
            format!(
                "{}\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:{}\nx-ms-version:2020-04-08\n{}",
                method, date, resource
            )
        } else {
            format!(
                "{}\n\n\n{}\n\n\n\n\n\n\n\n\nx-ms-date:{}\nx-ms-version:2020-04-08\n{}",
                method, content_length, date, resource
            )
        };

        // Decode account key and compute HMAC-SHA256 signature
        let key_bytes = general_purpose::STANDARD.decode(&self.account_key)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes)?;
        mac.update(string_to_sign.as_bytes());
        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        let auth_header = format!("SharedKey {}:{}", self.account_name, signature);
        Ok((auth_header, date))
    }

    /// Upload a blob to Azure given a full HTTPS URL.
    pub async fn put_object_from_url(&self, azure_url: &str, data: &[u8]) -> Result<()> {
        let (auth_header, date) = self.create_auth_header("PUT", azure_url, data.len())?;

        let response = self
            .client
            .put(azure_url)
            .header("Authorization", auth_header)
            .header("x-ms-date", date)
            .header("x-ms-version", "2020-04-08")
            .header("x-ms-blob-type", "BlockBlob")
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            bail!("Failed to upload blob: {} - {}", status, error_text);
        }

        Ok(())
    }
}

#[async_trait]
impl Connector for AzureConnector {
    /// Fetch a blob from Azure and return it as a `Read` stream.
    async fn fetch(&self, source: &str) -> Result<Box<dyn Read>> {
        let (auth_header, date) = self.create_auth_header("GET", source, 0)?;

        let response = self
            .client
            .get(source)
            .header("Authorization", auth_header)
            .header("x-ms-date", date)
            .header("x-ms-version", "2020-04-08")
            .send()
            .await?;

        if response.status().is_success() {
            let data = response.bytes().await?;
            Ok(Box::new(std::io::Cursor::new(data.to_vec())))
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            bail!("Failed to fetch blob: {} - {}", status, error_text);
        }
    }
}
