use super::Connector;
use anyhow::Result;
use std::io::Read;

pub struct SftpConnector;

impl SftpConnector {
	pub fn from_url(_url: &url::Url) -> Result<Self> {
		Ok(SftpConnector)
	}
}

#[async_trait::async_trait]
impl Connector for SftpConnector {
	fn scheme(&self) -> &'static str {
		"sftp"
	}

	async fn list(&self, _prefix: &str) -> Result<Vec<String>> {
		Ok(vec![])
	}

	async fn fetch(&self, _location: &str) -> Result<Box<dyn Read>> {
		Err(anyhow::anyhow!("SFTP connector not implemented"))
	}
}
