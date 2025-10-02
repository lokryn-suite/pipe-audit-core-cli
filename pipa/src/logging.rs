// Declare submodules
pub mod error;
pub mod ledger;
pub mod schema;
pub mod verify;
pub mod writer;

// Re-export the types and functions you want public
pub use ledger::ensure_ledger_key_exists;
pub use schema::AuditLogEntry;
pub use writer::log_event; // whatever types you define
pub fn init_logging() {
    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    ensure_ledger_key_exists()
}
