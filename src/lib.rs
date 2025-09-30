// src/lib.rs - Lokryn PipeAudit Core Library
// Company: Developyr
// Product: PipeAudit (part of Lokryn suite)

//! # PipeAudit
//!
//! Universal data validation framework for modern data engineering.
//! Part of the Lokryn data quality suite by Developyr.

// ===== STABLE PUBLIC API =====

/// Data validation contracts
pub mod contracts;

/// Validation execution and rules  
pub mod validators;

/// Error types
pub mod error;

/// Connection profiles
pub mod profiles;

/// Core validation runner (being refactored to core/)
pub mod runner;

// ===== EXTENSIBLE INTERFACES =====

/// Data source connectors
#[doc = "Interface for data source connectors (API may change before 1.0)"]
pub mod connectors;

/// File format drivers
#[doc = "Interface for file format drivers (API may change before 1.0)"]
pub mod drivers;

// ===== CORE BUSINESS LOGIC =====
/// Shared business logic used by both CLI and API
pub mod engine;

/// Storage and auth abstractions
pub mod traits;

// ===== INTERNAL MODULES =====

/// CLI interface - not part of stable API, may change
#[doc(hidden)]
pub mod cli;

/// CLI commands - not part of stable API, may change  
#[doc(hidden)]
pub mod commands;

/// Logging setup
pub mod logging;

// ===== API SERVER (feature gated) =====
#[cfg(feature = "api-server")]
pub mod api;

// ===== PRIMARY EXPORTS =====

pub use contracts::SchemaContracts;
pub use error::{ValidationError, ValidationResult};
pub use runner::validate_data;

// ===== FEATURES ========
#[cfg(feature = "file-management")]
pub mod movement;

// ===== VERSION INFO =====

/// PipeAudit version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Contract format version
pub const CONTRACT_VERSION: &str = "1.0";

/// Product info
pub const PRODUCT: &str = "Lokryn PipeAudit";
pub const COMPANY: &str = "Developyr";

// ===== PRELUDE =====

/// Common imports for users of PipeAudit
pub mod prelude {
    pub use crate::contracts::SchemaContracts;
    pub use crate::error::{ValidationError, ValidationResult};
    pub use crate::runner::validate_data;
    pub use crate::{COMPANY, CONTRACT_VERSION, PRODUCT, VERSION};
}
