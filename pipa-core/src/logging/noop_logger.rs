//! No-op logger implementation for testing.
//!
//! This logger discards all log entries. Useful for:
//! - Unit tests that don't need log output
//! - Benchmarking without I/O overhead
//! - Dry-run modes

use crate::logging::logger_trait::AuditLogger;
use crate::logging::schema::AuditLogEntry;

/// Logger that discards all log entries.
///
/// Use this in tests or when you need to run validations
/// without persisting audit logs.
pub struct NoOpLogger;

impl NoOpLogger {
    /// Create a new no-op logger.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger for NoOpLogger {
    fn log_event(&self, _entry: &AuditLogEntry) {
        // Intentionally do nothing
    }

    fn log_and_print(&self, _entry: &AuditLogEntry, console_msg: &str) {
        // Only print to console, don't log
        println!("{}", console_msg);
    }
}
