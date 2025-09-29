//! Contract storage abstraction

use async_trait::async_trait;
use crate::contracts::SchemaContracts;
use crate::error::ValidationResult;

#[async_trait]
pub trait ContractStore: Send + Sync {
    async fn list(&self) -> ValidationResult<Vec<String>>;
    async fn get(&self, name: &str) -> ValidationResult<SchemaContracts>;
    async fn exists(&self, name: &str) -> bool;
}