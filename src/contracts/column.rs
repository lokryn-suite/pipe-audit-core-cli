use serde::Deserialize;
use super::types::ContractType;

#[derive(Debug, Deserialize)]
pub struct ColumnContracts {
    pub name: String,
    pub contracts: Vec<ContractType>,
}
