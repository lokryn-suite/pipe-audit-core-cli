//! Google Cloud Storage (GCS) connector.
//!
//! Provides read/write access to GCS buckets using signed JWTs and the
//! REST API. Implements the generic `Connector` trait so it can be used
//! interchangeably with other backends (S3, Azure, local).
//!
//! ## Responsibilities
//! - Parse service account JSON from a profile.
//! - Generate OAuth2 access tokens via JWT bearer flow.
//! - Upload (`put_object_from_url`) and fetch (`fetch`) objects.
//! - Convert `gs://bucket/object` style URLs into REST API endpoints.
//!
//! ## Profile fields used
//! - `service_account_json` (must contain `client_email` and `private_key`)

use crate::connectors::Connector;
use crate::profiles::Profile;
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;
use std::io::Read;
use url::Url;

/// Concrete connector for GCS.
pub struct GCSConnector {
    client_email: String,
    private_key: String,
    client: reqwest::Client,
}

impl GCSConnector {
    /// Build a GCS connector from a profile and URL.
    ///
    /// Expects `service_account_json` in the profile.
    pub async fn from_profile_and_url(profile: &Profile, _url: &Url) -> Result<Self> {
        let service_account_json = profile
            .service_account_json
            .as_ref()
            .ok_or_else(|| anyhow!("GCS profile missing service_account_json"))?;

        let (client_email, private_key) = Self::parse_service_account(service_account_json)?;

        Ok(GCSConnector {
            client_email,
            private_key,
            client: reqwest::Client::new(),
        })
    }

    /// Parse service account JSON string into `(client_email, private_key)`.
    fn parse_service_account(service_account_json: &str) -> Result<(String, String)> {
        use serde_json::Value;

        let json: Value = serde_json::from_str(service_account_json)?;

        let client_email = json["client_email"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing client_email in service account JSON"))?
            .to_string();

        let private_key = json["private_key"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing private_key in service account JSON"))?
            .to_string();

        Ok((client_email, private_key))
    }

    /// Generate an OAuth2 access token using JWT bearer flow.
    ///
    /// - Signs a JWT with the service account private key.
    /// - Exchanges it for an access token at Google’s token endpoint.
    async fn generate_access_token(&self) -> Result<String> {
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "iss": self.client_email,
            "scope": "https://www.googleapis.com/auth/cloud-platform",
            "aud": "https://oauth2.googleapis.com/token",
            "exp": now + 3600,
            "iat": now
        });

        let header = Header::new(Algorithm::RS256);
        let encoding_key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())?;
        let jwt_token = encode(&header, &claims, &encoding_key)?;

        let token_response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt_token),
            ])
            .send()
            .await?;

        if !token_response.status().is_success() {
            bail!("Token exchange failed: {}", token_response.status());
        }

        let token_json: serde_json::Value = token_response.json().await?;
        let access_token = token_json["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("access_token not found in response"))?
            .to_string();

        Ok(access_token)
    }

    /// Convert a `gs://bucket/object` style URL into a REST API endpoint.
    fn convert_to_rest_api_url(&self, source_url: &str) -> Result<String> {
        let url = Url::parse(source_url)?;
        let path = url.path().trim_start_matches('/');
        let parts: Vec<&str> = path.splitn(2, '/').collect();

        if parts.len() != 2 {
            bail!("Invalid GCS URL format");
        }

        let bucket = parts[0];
        let object_path = urlencoding::encode(parts[1]);

        Ok(format!(
            "https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
            bucket, object_path
        ))
    }

    /// Upload an object to GCS given a `gs://bucket/object` URL.
    pub async fn put_object_from_url(&self, gcs_url: &str, data: &[u8]) -> Result<()> {
        let access_token = self.generate_access_token().await?;
        let api_url = self
            .convert_to_rest_api_url(gcs_url)?
            .replace("?alt=media", ""); // remove download flag for upload

        let response = self
            .client
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            bail!("Failed to upload object: {} - {}", status, error_text);
        }

        Ok(())
    }
}

#[async_trait]
impl Connector for GCSConnector {
    /// Fetch an object from GCS and return it as a `Read` stream.
    async fn fetch(&self, source: &str) -> Result<Box<dyn Read>> {
        let access_token = self.generate_access_token().await?;
        let api_url = self.convert_to_rest_api_url(source)?;

        let response = self
            .client
            .get(&api_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            let data = response.bytes().await?;
            Ok(Box::new(std::io::Cursor::new(data.to_vec())))
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            bail!("Failed to fetch object: {} - {}", status, error_text);
        }
    }
}
