// Declare submodules
pub mod error;
pub mod ledger;
pub mod schema;
pub mod verify;
pub mod writer;

// Re-export the types and functions you want public
pub use crate::logging::verify::{verify_all, verify_date};
pub use error::{ValidationError, ValidationResult};
pub use ledger::{append_to_ledger, ensure_ledger_key_exists, read_ledger_plaintext};
pub use schema::{AuditLogEntry, Contract, Executor, ProcessSummary, RuleResult, Target};
pub use writer::log_event; // whatever types you define
pub fn init_logging() {
    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    ensure_ledger_key_exists()
}
