use super::types::ContractType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ColumnContracts {
    pub name: String,
    pub contracts: Vec<ContractType>,
}
