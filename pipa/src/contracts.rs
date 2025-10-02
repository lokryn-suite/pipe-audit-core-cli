// Submodules that define different contract domains
pub mod column;    // Column-level constraints (type, nullability, length, etc.)
pub mod compound;  // Multi-column constraints (e.g., uniqueness across fields)
pub mod file;      // File-level constraints (row counts, completeness)
pub mod schema;    // Schema definitions and contract orchestration
pub mod types;     // Shared enums and type definitions for contracts

// Curated re-exports: the stable API surface for contracts
pub use schema::{SchemaContracts, load_contract_for_file};
pub use types::ContractType;
