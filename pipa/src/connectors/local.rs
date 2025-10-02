use super::Connector;
use anyhow::Result;
use std::fs::File;
use std::io::Read;

pub struct LocalConnector;

impl LocalConnector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Connector for LocalConnector {
    async fn fetch(&self, path: &str) -> Result<Box<dyn Read>> {
        Ok(Box::new(File::open(path)?))
    }
}
