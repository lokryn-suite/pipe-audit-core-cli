use super::types::ContractType;
use serde::Deserialize;

/// Column-level contract definition.
///
/// Each `ColumnContracts` entry binds a column name to one or more
/// validation rules (`ContractType`). These rules are deserialized
/// from TOML/JSON/YAML contract files.
///
/// Example TOML:
/// ```toml
/// [[columns]]
/// name = "email"
/// validation = [
///   { rule = "not_null" },
///   { rule = "pattern", pattern = "^[^@]+@[^@]+$" }
/// ]
/// ```
#[derive(Debug, Deserialize)]
pub struct ColumnContracts {
    /// The column name in the dataset to which these rules apply.
    pub name: String,

    /// A list of validation rules to enforce on this column.
    /// Each rule is a `ContractType` variant (e.g., NotNull, Pattern, MaxLength).
    pub validation: Vec<ContractType>,
}
