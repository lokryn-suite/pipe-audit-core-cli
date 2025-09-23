use serde::Deserialize;
use super::{column::ColumnContracts, file::FileContracts, compound::CompoundUnique};

#[derive(Debug, Deserialize)]
pub struct SchemaContracts {
    pub name: String,
    pub version: String,
    pub file: Option<FileContracts>,  
    pub columns: Vec<ColumnContracts>,
    pub compound_unique: Option<Vec<CompoundUnique>>,
}
