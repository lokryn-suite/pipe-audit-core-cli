use super::types::ContractType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ColumnContracts {
    pub name: String,
    pub validation: Vec<ContractType>,
}
