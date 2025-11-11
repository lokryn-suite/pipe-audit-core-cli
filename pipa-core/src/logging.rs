// Declare submodules
pub(crate) mod error;
pub(crate) mod init;
pub(crate) mod ledger;
pub(crate) mod schema;
pub(crate) mod verify;
pub(crate) mod writer;

// New: pluggable logging infrastructure
pub(crate) mod logger_trait;
pub(crate) mod jsonl_logger;
pub(crate) mod noop_logger;

// Re-export the types and functions you want public
pub(crate) use schema::AuditLogEntry;
#[allow(unused_imports)]
pub(crate) use writer::log_event; // Deprecated, will be removed in favor of trait

// Public exports for the new logging trait system
pub use logger_trait::AuditLogger;
pub use jsonl_logger::JsonlLogger;
pub use noop_logger::NoOpLogger;
