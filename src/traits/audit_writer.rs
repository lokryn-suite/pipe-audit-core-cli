//! Audit log writer abstraction

use async_trait::async_trait;
use crate::logging::schema::AuditLogEntry;
use crate::error::ValidationResult;

#[async_trait]
pub trait AuditWriter: Send + Sync {
    async fn write(&self, entry: &AuditLogEntry) -> ValidationResult<()>;
}