//! Trait abstractions for storage and auth

pub mod audit_writer;
pub mod auth;
pub mod contract_store;

pub use audit_writer::AuditWriter;
pub use auth::{AuthContext, NoOpAuth, User};
pub use contract_store::ContractStore;
