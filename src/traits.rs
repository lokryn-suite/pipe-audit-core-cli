//! Trait abstractions for storage and auth

pub mod auth;
pub mod contract_store;
pub mod audit_writer;

pub use auth::{AuthContext, NoOpAuth, User};
pub use contract_store::ContractStore;
pub use audit_writer::AuditWriter;