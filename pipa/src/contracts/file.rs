use super::types::ContractType;
use serde::Deserialize;

/// File-level contract definition.
///
/// Associates a set of validation rules (`ContractType`) with an entire file.
/// These rules apply to the dataset as a whole, rather than individual columns.
///
/// Example TOML:
/// ```toml
/// [file]
/// validation = [
///   { rule = "row_count", min = 100, max = 200 },
///   { rule = "completeness", min_ratio = 0.95 }
/// ]
/// ```
#[derive(Debug, Deserialize)]
pub struct FileContracts {
    /// A list of validation rules to enforce at the file level.
    /// Examples: RowCount, Completeness, Exists.
    pub validation: Vec<ContractType>,
}
