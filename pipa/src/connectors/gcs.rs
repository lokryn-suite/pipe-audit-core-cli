// src/connectors/gcs.rs
use crate::connectors::Connector;
use crate::profiles::Profile;
use anyhow::Result;
use async_trait::async_trait;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
use std::io::Read;
use url::Url;

pub struct GCSConnector {
    client_email: String,
    private_key: String,
    client: reqwest::Client,
}

impl GCSConnector {
    pub async fn from_profile_and_url(profile: &Profile, _url: &Url) -> Result<Self> {
        let service_account_json = profile
            .service_account_json
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GCS profile missing service_account_json"))?;

        let (client_email, private_key) = Self::parse_service_account(service_account_json)?;

        Ok(GCSConnector {
            client_email,
            private_key,
            client: reqwest::Client::new(),
        })
    }

    fn parse_service_account(service_account_json: &str) -> Result<(String, String)> {
        use serde_json::Value;

        let json: Value = serde_json::from_str(service_account_json)?;

        let client_email = json["client_email"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing client_email in service account JSON"))?
            .to_string();

        let private_key = json["private_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing private_key in service account JSON"))?
            .to_string();

        Ok((client_email, private_key))
    }

    async fn generate_access_token(&self) -> Result<String> {
        // Create JWT claims
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "iss": self.client_email,
            "scope": "https://www.googleapis.com/auth/cloud-platform",
            "aud": "https://oauth2.googleapis.com/token",
            "exp": now + 3600, // 1 hour
            "iat": now
        });

        // Generate JWT token
        let header = Header::new(Algorithm::RS256);
        let encoding_key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())?;
        let jwt_token = encode(&header, &claims, &encoding_key)?;

        // Exchange JWT for access token
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
            return Err(anyhow::anyhow!(
                "Token exchange failed: {}",
                token_response.status()
            ));
        }

        let token_json: serde_json::Value = token_response.json().await?;
        let access_token = token_json["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("access_token not found in response"))?
            .to_string();

        Ok(access_token)
    }

    fn convert_to_rest_api_url(&self, source_url: &str) -> Result<String> {
        // Convert https://storage.cloud.google.com/bucket/path
        // to https://storage.googleapis.com/storage/v1/b/bucket/o/path?alt=media

        let url = Url::parse(source_url)?;
        let path = url.path().trim_start_matches('/');
        let parts: Vec<&str> = path.splitn(2, '/').collect();

        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid GCS URL format"));
        }

        let bucket = parts[0];
        let object_path = urlencoding::encode(parts[1]);

        Ok(format!(
            "https://storage.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
            bucket, object_path
        ))
    }

    pub async fn put_object_from_url(&self, gcs_url: &str, data: &[u8]) -> Result<()> {
        let access_token = self.generate_access_token().await?;
        let api_url = self
            .convert_to_rest_api_url(gcs_url)?
            .replace("?alt=media", ""); // Remove alt=media for upload

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
            return Err(anyhow::anyhow!(
                "Failed to upload object: {} - {}",
                status,
                error_text
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl Connector for GCSConnector {
    fn scheme(&self) -> &'static str {
        "https"
    }

    async fn list(&self, _prefix: &str) -> Result<Vec<String>> {
        // Return empty for now - implement if needed
        Ok(vec![])
    }

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
            Err(anyhow::anyhow!(
                "Failed to fetch object: {} - {}",
                status,
                error_text
            ))
        }
    }
}
