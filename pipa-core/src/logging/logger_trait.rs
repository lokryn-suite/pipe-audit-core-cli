//! Trait definition for pluggable audit logging.
//!
//! This trait allows consumers of `pipa-core` to provide their own
//! logging implementation (e.g., JSONL files, DuckDB, in-memory, etc.).

use crate::logging::schema::AuditLogEntry;

/// Trait for audit logging implementations.
///
/// Implementors can store audit logs in any backend they choose:
/// - JSONL files (default CLI behavior)
/// - DuckDB (for containerized service)
/// - In-memory (for testing)
/// - Remote logging services
///
/// The trait is `Send + Sync` to allow usage in async contexts
/// and across threads.
pub trait AuditLogger: Send + Sync {
    /// Log an audit event.
    ///
    /// This is the core logging method. Implementations should:
    /// - Serialize the entry to their chosen format
    /// - Store it in their backend
    /// - Handle any errors internally (or panic if unrecoverable)
    fn log_event(&self, entry: &AuditLogEntry);

    /// Log an audit event and print a console message.
    ///
    /// This is a convenience method for CLI-style usage where you want
    /// both structured logging and human-readable console output.
    ///
    /// Default implementation calls `log_event()` then prints to stdout.
    fn log_and_print(&self, entry: &AuditLogEntry, console_msg: &str) {
        self.log_event(entry);
        println!("{}", console_msg);
    }
}
