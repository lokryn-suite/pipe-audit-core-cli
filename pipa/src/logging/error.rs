use polars::prelude::PolarsError;
use thiserror::Error;

/// Main error type for PipeAudit
///
/// Centralizes all error cases that can occur during validation,
/// data loading, contract parsing, or connector operations.
/// Uses `thiserror` for ergonomic `Display` + `Error` implementations.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Filesystem or IO failure
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Polars DataFrame operation failed
    #[error("Polars operation failed: {0}")]
    Polars(#[from] PolarsError),

    /// Contract TOML parsing error
    #[error("Contract parsing error: {0}")]
    ContractParse(String),

    /// Validation logic failed (semantic error, not just parse)
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Connector (S3, GCS, etc.) error
    #[error("Connector error: {0}")]
    Connector(String),

    /// Profile lookup failed
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    /// Regex compilation or execution error
    #[error("Regex pattern error: {0}")]
    Regex(#[from] regex::Error),

    /// Catch-all for internal errors wrapped in `anyhow`
    #[error("Internal Error: {0}")]
    Anyhow(#[from] anyhow::Error),

    /// File size exceeded configured maximum
    #[error("File size {size} exceeds maximum {max} bytes")]
    FileTooLarge { size: usize, max: usize },

    /// Generic fallback error
    #[error("{0}")]
    Other(String),
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Blanket conversion from boxed errors into `ValidationError::Other`.
/// This ensures any dynamic error can be folded into the unified error type.
impl From<Box<dyn std::error::Error>> for ValidationError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ValidationError::Other(err.to_string())
    }
}
