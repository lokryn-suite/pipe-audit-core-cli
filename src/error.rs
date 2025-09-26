// src/error.rs (create if doesn't exist)
use polars::prelude::PolarsError;
use thiserror::Error;

/// Main error type for PipeAudit
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Polars operation failed: {0}")]
    Polars(#[from] PolarsError),

    #[error("Contract parsing error: {0}")]
    ContractParse(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Connector error: {0}")]
    Connector(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Regex pattern error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Validation failed: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("{0}")]
    Other(String),
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;
