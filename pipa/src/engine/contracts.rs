// Submodules that implement contract execution logic
pub mod meta; // Metadata + contract lookup/listing/validation
pub mod runner; // Execution engine for running validations

// Curated re-exports: the stable API surface for engine contracts
pub use meta::{
    ContractInfo,       // Metadata about a contract (name, version, etc.)
    ContractList,       // Collection of available contracts
    ContractValidation, // Result of validating a contract
    get_contract,       // Lookup a single contract by name/path
    list_contracts,     // Enumerate all available contracts
    validate_contract,  // Validate a contract definition (schema-level check)
};

pub use runner::{
    ValidationOutcome,       // Result of executing a contract against data
    run_contract_validation, // Entry point to run validations
};
