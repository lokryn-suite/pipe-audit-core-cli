use super::types::ContractType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FileContracts {
    pub validation: Vec<ContractType>,
}
