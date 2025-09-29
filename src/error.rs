// src/error.rs
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

    #[error("Internal Error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("File size {size} exceeds maximum {max} bytes")]
    FileTooLarge { size: usize, max: usize },

    #[error("{0}")]
    Other(String),
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

// Add this at the end:
impl From<Box<dyn std::error::Error>> for ValidationError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ValidationError::Other(err.to_string())
    }
}
