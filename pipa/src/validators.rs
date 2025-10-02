//! Validator traits and common types.
//!
//! This module defines the core traits (`Validator`, `FileValidator`,
//! `CompoundValidator`) that all rule implementations must conform to.
//! It also provides the `ValidationReport` struct, which is the standard
//! return type for all validators.
//!
//! Submodules:
//! - `column`: column-level validators (NotNull, Unique, Pattern, etc.)
//! - `file`: file-level validators (RowCount, Completeness, etc.)
//! - `compound`: multi-column validators (CompoundUnique, etc.)
//!
//! The engine (`engine/validation.rs`) dispatches to these traits based
//! on the `ContractType` enum. Each validator is responsible for
//! implementing its own logic against a Polars `DataFrame`.

use crate::logging::error::ValidationResult;
use polars::prelude::*;

// -----------------------------------------------------------------------------
// Sub-modules for different validator categories
// -----------------------------------------------------------------------------
pub mod column;
pub mod compound;
pub mod file;

// -----------------------------------------------------------------------------
// Common types
// -----------------------------------------------------------------------------

/// Standardized result of a validation check.
///
/// - `status`: one of `"pass"`, `"fail"`, or `"skipped"`.
/// - `details`: optional human-readable explanation (e.g., failure reason).
///
/// This struct is converted into a `RuleResult` for audit logging.
#[derive(Debug)]
pub struct ValidationReport {
    pub status: &'static str,
    pub details: Option<String>,
}

// -----------------------------------------------------------------------------
// Core traits
// -----------------------------------------------------------------------------

/// Trait for all **column-level validators**.
///
/// Implementors validate a single column of a DataFrame against a rule.
/// Example: `NotNullValidator`, `PatternValidator`.
pub trait Validator {
    /// Human-readable name of the validator (e.g., `"not_null"`).
    fn name(&self) -> &'static str;

    /// Apply the validation to the given column of the DataFrame.
    ///
    /// # Arguments
    /// * `df` - The full DataFrame.
    /// * `column_name` - The name of the column to validate.
    ///
    /// # Returns
    /// * `ValidationReport` with status and optional details.
    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport>;
}

/// Trait for all **file-level validators**.
///
/// Implementors validate properties of the entire DataFrame.
/// Example: `RowCountValidator`, `FileCompletenessValidator`.
pub trait FileValidator {
    /// Human-readable name of the validator.
    fn name(&self) -> &'static str;

    /// Apply the validation to the entire DataFrame.
    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport>;
}

/// Trait for all **compound (multi-column) validators**.
///
/// Implementors validate relationships across multiple columns.
/// Example: `CompoundUniqueValidator`.
pub trait CompoundValidator {
    /// Human-readable name of the validator.
    fn name(&self) -> &'static str;

    /// Apply the validation across multiple columns of the DataFrame.
    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport>;
}
