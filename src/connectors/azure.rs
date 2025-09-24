use super::Connector;
use anyhow::Result;
use std::io::Read;

pub struct AzureConnector;

impl AzureConnector {
	pub fn from_url(_url: &url::Url) -> Result<Self> {
		Ok(AzureConnector)
	}
}

#[async_trait::async_trait]
impl Connector for AzureConnector {
	fn scheme(&self) -> &'static str {
		"azure"
	}

	async fn list(&self, _prefix: &str) -> Result<Vec<String>> {
		Ok(vec![])
	}

	async fn fetch(&self, _location: &str) -> Result<Box<dyn Read>> {
		Err(anyhow::anyhow!("Azure connector not implemented"))
	}
}
