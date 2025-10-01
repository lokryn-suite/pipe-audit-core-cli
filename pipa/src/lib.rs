//! # Pipa
//!
//! Public API surface for the Pipa data quality engine.
//!
//! This crate is primarily used via the CLI, but the same functionality
//! is available programmatically through the modules below.
//!
//! Internals (engine, connectors, movement, validators, etc.) are kept private
//! so they can evolve without breaking consumers.

mod connectors;
mod contracts;
mod drivers;
mod engine;
mod logging;
mod movement;
mod profiles;
mod runner;
mod traits;
mod validators;

// -----------------------------
// Public API surface
// -----------------------------

/// Contract management: list, validate, show, and run contracts.
pub mod contract {
    pub use crate::engine::contracts::{
        ContractInfo, ContractList, ContractValidation, ValidationOutcome,
        get_contract, list_contracts, run_contract_validation, validate_contract
    };
    pub use crate::logging::schema::Executor;
}

/// Profile management: list and test profiles.
pub mod profile {
    pub use crate::engine::profiles::{
        ProfileList, ProfileTestResult, list_profiles, test_profile,
    };
}

/// Run data validation against contracts.
pub mod run {
    pub use crate::engine::contracts::run_contract_validation;
}

/// Log management: verify log integrity.
pub mod logs {
    pub use crate::engine::logs::{LogVerification, verify_logs};
    pub use crate::logging::verify::FileStatus;
    pub use crate::logging::init_logging;
}

/// System health checks.
pub mod health {
    pub use crate::engine::system::{HealthStatus, check_system_health, run_health_check};
}

/// Initialize project scaffolding.
pub mod init {
    pub use crate::engine::init::init_project;
}
