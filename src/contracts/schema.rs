use serde::Deserialize;
use std::path::Path;

use super::{column::ColumnContracts, compound::CompoundUnique, file::FileContracts};

#[derive(Debug, Deserialize)]
pub struct SchemaContracts {
    pub name: String,
    pub version: String,
    pub file: Option<FileContracts>,
    pub columns: Vec<ColumnContracts>,
    pub compound_unique: Option<Vec<CompoundUnique>>,
}

/// Load the TOML contract file that matches the data filename
pub fn load_contract_for_file(path: &Path) -> SchemaContracts {
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let contract_path = format!("contracts/{}.toml", stem);

    let toml_str = std::fs::read_to_string(&contract_path)
        .unwrap_or_else(|_| panic!("Missing contract file: {}", contract_path));

    toml::from_str(&toml_str).expect("Failed to parse contract TOML")
}
