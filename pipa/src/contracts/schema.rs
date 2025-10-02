use serde::Deserialize;
use std::path::Path;

use super::{column::ColumnContracts, compound::CompoundUnique, file::FileContracts};

/// High-level metadata about a contract.
/// 
/// - `name` and `version` identify the contract.
/// - `tags` can be used for grouping or filtering, but are currently unused.
#[derive(Debug, Deserialize)]
pub struct Contract {
    pub name: String,
    pub version: String,
    #[allow(dead_code)]
    pub tags: Vec<String>,
}

/// Input source definition for a contract.
/// 
/// - `type`: connector type (e.g., "s3", "local").
/// - `location`: path/URI to the data.
/// - `profile`: optional profile name for credentials/config.
#[derive(Debug, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub r#type: String,
    pub location: Option<String>,
    pub profile: Option<String>,
}

/// Output destination definition.
/// 
/// Similar to `Source`, but may include a `format` override
/// (e.g., force output as CSV even if input was Parquet).
#[derive(Debug, Deserialize, Clone)]
pub struct Destination {
    #[serde(rename = "type")]
    pub r#type: String,
    pub location: Option<String>,
    pub profile: Option<String>,
    pub format: Option<String>,
}

/// Quarantine sink definition.
/// 
/// Used to redirect invalid rows/files. Mirrors `Destination`
/// but semantically distinct.
#[derive(Debug, Deserialize, Clone)]
pub struct Quarantine {
    #[serde(rename = "type")]
    pub r#type: String,
    pub location: Option<String>,
    pub profile: Option<String>,
    pub format: Option<String>,
}

/// The full schema contract definition.
/// 
/// This is the top-level structure deserialized from a TOML file.
/// It aggregates all contract components:
/// - `contract`: metadata
/// - `file`: file-level rules
/// - `columns`: column-level rules
/// - `compound_unique`: multi-column uniqueness rules
/// - `source`, `destination`, `quarantine`: I/O configuration
#[derive(Debug, Deserialize)]
pub struct SchemaContracts {
    pub contract: Contract,
    pub file: Option<FileContracts>,
    pub columns: Vec<ColumnContracts>,
    pub compound_unique: Option<Vec<CompoundUnique>>,
    pub source: Option<Source>,
    pub destination: Option<Destination>, 
    pub quarantine: Option<Quarantine>,   
}

/// Load the TOML contract file that matches the data filename.
///
/// - Derives the contract filename from the data file stem.
/// - Reads `contracts/{stem}.toml`.
/// - Panics if the file is missing or invalid.
///
/// Example:
/// ```ignore
/// let schema = load_contract_for_file(Path::new("people.csv"));
/// // loads "contracts/people.toml"
/// ```
pub fn load_contract_for_file(path: &Path) -> SchemaContracts {
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let contract_path = format!("contracts/{}.toml", stem);

    let toml_str = std::fs::read_to_string(&contract_path)
        .unwrap_or_else(|_| panic!("Missing contract file: {}", contract_path));

    toml::from_str(&toml_str).expect("Failed to parse contract TOML")
}
