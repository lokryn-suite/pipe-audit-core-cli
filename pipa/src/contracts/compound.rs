use serde::Deserialize;

/// Compound uniqueness contract.
///
/// Ensures that the combination of values across multiple columns
/// is unique within the dataset. This is the multi-column equivalent
/// of a single-column `Unique` rule.
///
/// Example TOML:
/// ```toml
/// [[compound_unique]]
/// columns = ["first_name", "last_name", "dob"]
/// ```
///
/// This would enforce that no two rows share the same
/// (first_name, last_name, dob) triple.
#[derive(Debug, Deserialize)]
pub struct CompoundUnique {
    /// The set of columns that must be unique in combination.
    pub columns: Vec<String>,
}
