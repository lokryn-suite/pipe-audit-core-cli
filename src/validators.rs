// src/validators.rs

use crate::error::ValidationResult;
use polars::prelude::*;

// Sub-modules for different validator types
pub mod column;
pub mod compound;
pub mod file;

// A standard struct to carry the result of a validation check.
#[derive(Debug)]
pub struct ValidationReport {
    pub status: &'static str, // "pass", "fail", or "skipped"
    pub details: Option<String>,
}

/// The core trait that all individual COLUMN validators will implement.
pub trait Validator {
    fn name(&self) -> &'static str;
    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport>;
}

/// The trait for all FILE-LEVEL validators.
pub trait FileValidator {
    fn name(&self) -> &'static str;
    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport>;
}

/// The trait for all COMPOUND (multi-column) validators.
pub trait CompoundValidator {
    fn name(&self) -> &'static str;
    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport>;
}
