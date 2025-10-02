use super::Connector;
use anyhow::Result;
use std::io::Read;

#[allow(dead_code)]
pub struct SftpConnector;

impl SftpConnector {
    #[allow(dead_code)]
    pub fn from_url(_url: &url::Url) -> Result<Self> {
        Ok(SftpConnector)
    }
}

#[async_trait::async_trait]
impl Connector for SftpConnector {
    async fn fetch(&self, _location: &str) -> Result<Box<dyn Read>> {
        Err(anyhow::anyhow!("SFTP connector not implemented"))
    }
}
