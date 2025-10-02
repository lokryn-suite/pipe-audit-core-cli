// Declare submodules
pub(crate) mod error;
pub(crate) mod init;
pub(crate) mod ledger;
pub(crate) mod schema;
pub(crate) mod verify;
pub(crate) mod writer;

// Re-export the types and functions you want public
pub(crate) use schema::AuditLogEntry;
pub(crate) use writer::log_event; // whatever types you define
