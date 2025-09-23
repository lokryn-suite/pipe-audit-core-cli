use serde::Deserialize;
use super::types::ContractType;

#[derive(Debug, Deserialize)]
pub struct FileContracts {
    pub contracts: Vec<ContractType>,
}
