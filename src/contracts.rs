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
}

#[derive(Debug, Deserialize)]
pub struct ColumnContracts {
    pub name: String,
    pub contracts: Vec<ContractType>,
}

#[derive(Debug, Deserialize)]
pub struct TableContracts {
    pub contracts: Vec<ContractType>,
}

#[derive(Debug, Deserialize)]
pub struct CompoundUnique {
    pub columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileContracts {
    pub table: Option<TableContracts>,
    pub columns: Vec<ColumnContracts>,
    pub compound_unique: Option<Vec<CompoundUnique>>,
}
