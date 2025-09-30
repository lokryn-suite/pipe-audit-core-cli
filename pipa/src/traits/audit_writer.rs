//! Audit log writer abstraction

use crate::logging::error::ValidationResult;
use crate::logging::schema::AuditLogEntry;
use async_trait::async_trait;

#[async_trait]
pub trait AuditWriter: Send + Sync {
    async fn write(&self, entry: &AuditLogEntry) -> ValidationResult<()>;
}
