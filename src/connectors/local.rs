use super::Connector;
use anyhow::Result;
use glob::glob;
use std::fs::File;
use std::io::Read;

pub struct LocalConnector;

impl LocalConnector {
    pub fn new() -> Self {
        Self
    }
}

impl Connector for LocalConnector {
    fn scheme(&self) -> &'static str {
        "file"
    }

    fn list(&self, pattern: &str) -> Result<Vec<String>> {
        let mut files = Vec::new();
        for entry in glob(pattern)? {
            files.push(entry?.to_string_lossy().to_string());
        }
        Ok(files)
    }

    fn fetch(&self, path: &str) -> Result<Box<dyn Read>> {
        Ok(Box::new(File::open(path)?))
    }
}
