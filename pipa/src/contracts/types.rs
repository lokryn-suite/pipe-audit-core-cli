use serde::Deserialize;

/// Enumeration of all supported contract rules.
///
/// Each variant corresponds to a validation rule that can be declared
/// in a contract file (TOML/JSON/YAML). `serde` is used to deserialize
/// user-specified rules into strongly typed Rust enums.
///
/// The `#[serde(tag = "rule", rename_all = "snake_case")]` attribute means:
/// - Contracts must specify a field `"rule"` with the variant name.
/// - Variant names are expected in snake_case (e.g., `not_null`, `max_length`).
#[derive(Debug, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum ContractType {
    // Column-level rules
    NotNull,
    Unique,
    Pattern { pattern: String },
    MaxLength { value: usize },
    Range { min: i64, max: i64 },
    InSet { values: Vec<String> },
    NotInSet { values: Vec<String> },
    Boolean,
    Type { dtype: String },
    DateFormat { format: String },

    // Statistical rules
    OutlierSigma { sigma: f64 },
    Distinctness { min_ratio: f64 },
    Completeness { min_ratio: f64 },
    MeanBetween { min: f64, max: f64 },
    StdevBetween { min: f64, max: f64 },

    // File-level rules
    RowCount { min: usize, max: Option<usize> },
    Exists,

    // Experimental / unused rules
    #[allow(dead_code)]
    MinBetween { min: i64, max: i64 },
    #[allow(dead_code)]
    MaxBetween { min: i64, max: i64 },
}
