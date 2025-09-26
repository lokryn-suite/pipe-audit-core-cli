use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum ContractType {
    NotNull,
    Unique,
    Pattern { pattern: String },
    MaxLength { value: usize },
    Range { min: i64, max: i64 },
    InSet { values: Vec<String> },
    Boolean,
    OutlierSigma { sigma: f64 },
    Distinctness { min_ratio: f64 },
    Completeness { min_ratio: f64 },
    RowCount { min: usize, max: Option<usize> },
    NotInSet { values: Vec<String> },
    Exists,
    Type { dtype: String },
    MinBetween { min: i64, max: i64 },
    MaxBetween { min: i64, max: i64 },
    MeanBetween { min: f64, max: f64 },
    StdevBetween { min: f64, max: f64 },
    DateFormat { format: String },
}
