use super::Connector;
use anyhow::Result;
use std::io::Read;

pub struct S3Connector;

impl S3Connector {
	pub fn from_url(_url: &url::Url) -> Result<Self> {
		// Minimal stub — real implementation omitted
		Ok(S3Connector)
	}
}

impl Connector for S3Connector {
	fn scheme(&self) -> &'static str {
		"s3"
	}

	fn list(&self, _prefix: &str) -> Result<Vec<String>> {
		Ok(vec![])
	}

	fn fetch(&self, _location: &str) -> Result<Box<dyn Read>> {
		Err(anyhow::anyhow!("S3 connector not implemented"))
	}
}
