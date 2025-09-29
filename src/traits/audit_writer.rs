//! Audit log writer abstraction

use crate::error::ValidationResult;
use crate::logging::schema::AuditLogEntry;
use async_trait::async_trait;

#[async_trait]
pub trait AuditWriter: Send + Sync {
    async fn write(&self, entry: &AuditLogEntry) -> ValidationResult<()>;
}
