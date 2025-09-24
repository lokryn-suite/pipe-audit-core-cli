use super::Connector;
use anyhow::Result;
use std::io::Read;

pub struct GcsConnector;

impl GcsConnector {
	pub fn from_url(_url: &url::Url) -> Result<Self> {
		Ok(GcsConnector)
	}
}

#[async_trait::async_trait]
impl Connector for GcsConnector {
	fn scheme(&self) -> &'static str {
		"gcs"
	}

	async fn list(&self, _prefix: &str) -> Result<Vec<String>> {
		Ok(vec![])
	}

	async fn fetch(&self, _location: &str) -> Result<Box<dyn Read>> {
		Err(anyhow::anyhow!("GCS connector not implemented"))
	}
}
