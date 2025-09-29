//! Authentication context trait
//! Stub implementation for Docker, real implementation for Cloud

use crate::error::ValidationResult;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub tenant_id: Option<String>,
}

#[async_trait]
pub trait AuthContext: Send + Sync {
    async fn authenticate(&self, token: &str) -> ValidationResult<User>;
}

/// No-op auth for Docker free tier
pub struct NoOpAuth;

#[async_trait]
impl AuthContext for NoOpAuth {
    async fn authenticate(&self, _token: &str) -> ValidationResult<User> {
        Ok(User {
            id: "local".to_string(),
            tenant_id: None,
        })
    }
}
