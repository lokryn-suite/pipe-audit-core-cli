//! # Pipa
//!
//! Public API surface for the Pipa data quality engine.
//!
//! This crate is primarily used via the CLI, but the same functionality
//! is available programmatically through the modules below.
//!
//! Internals (engine, connectors, movement, validators, etc.) are kept private
//! so they can evolve without breaking consumers.

// --- Internal modules (not re-exported directly) ---
// These are the building blocks of the engine. They remain private
// so their APIs can change without breaking downstream users.
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
//
// Each public submodule below re-exports a curated set of types and
// functions from the internal engine. This creates a stable, branded
// API for consumers (CLI or programmatic) while insulating them from
// internal refactors.

/// Contract management: list, validate, show, and run contracts.
///
/// Exposes contract-related types and functions from `engine::contracts`.
/// Also re-exports the `Executor` type from logging for contract execution context.
pub mod contract {
    pub use crate::engine::contracts::{
        ContractInfo, ContractList, ContractValidation, ValidationOutcome,
        get_contract, list_contracts, run_contract_validation, validate_contract
    };
    pub use crate::logging::schema::Executor;
}

/// Profile management: list and test profiles.
///
/// Provides access to profile definitions and testing utilities.
/// Profiles are typically used to parameterize contract runs.
pub mod profile {
    pub use crate::engine::profiles::{
        ProfileList, ProfileTestResult, list_profiles, test_profile,
    };
}

/// Run data validation against contracts.
///
/// Thin wrapper that exposes the core validation runner directly.
/// Useful for programmatic invocation without going through CLI.
pub mod run {
    pub use crate::engine::contracts::run_contract_validation;
}

/// Log management: verify log integrity.
///
/// Surfaces log verification and integrity checking.
/// Includes cryptographic verification of log chains.
pub mod logs {
    pub use crate::engine::logs::{LogVerification, verify_logs};
    pub use crate::logging::verify::FileStatus;
    pub use crate::logging::init_logging;
}

/// System health checks.
///
/// Provides system-level diagnostics (e.g., environment, connectors).
/// Useful for pre-flight checks before running validations.
pub mod health {
    pub use crate::engine::system::{HealthStatus, check_system_health, run_health_check};
}

/// Initialize project scaffolding.
///
/// Exposes project initialization helpers (e.g., creating config files,
/// setting up directories). Typically used by `pipa init`.
pub mod init {
    pub use crate::engine::init::init_project;
}
