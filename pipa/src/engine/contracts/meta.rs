//! Contract metadata and syntax validation functions

use crate::contracts::load_contract_for_file; // loads and parses TOML into SchemaContracts
use crate::engine::log_action;                // audit logging hook
use glob;                                     // filesystem globbing
use std::path::Path;

/// Result of listing contracts
pub struct ContractList {
    pub contracts: Vec<String>, // contract names (file stems)
}

/// Result of getting a contract
pub struct ContractInfo {
    pub name: String,   // contract name
    pub version: String,// version string from contract metadata
    pub exists: bool,   // whether the contract file exists
}

/// Result of validating a contract
pub struct ContractValidation {
    pub valid: bool,          // true if contract parsed successfully
    pub error: Option<String>,// error message if invalid
}

/// List all available contracts by scanning `contracts/*.toml`.
/// Returns both the list and a log message.
pub fn list_contracts() -> Result<(ContractList, String), String> {
    let contracts: Vec<String> = match glob::glob("contracts/*.toml") {
        Ok(paths) => paths
            .filter_map(Result::ok)
            .filter_map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect(),
        Err(_) => return Err("Failed to read contracts directory".to_string()),
    };

    let message = log_action("contracts_listed", None, None, None, None);
    Ok((ContractList { contracts }, message))
}

/// Get information about a specific contract.
/// Returns metadata (name, version, exists) and a log message.
pub fn get_contract(name: &str) -> (ContractInfo, String) {
    let contract_path = format!("contracts/{}.toml", name);

    if !Path::new(&contract_path).exists() {
        let message = log_action(
            "contract_retrieved",
            Some("exists=false"),
            Some(name),
            None,
            None,
        );
        return (
            ContractInfo {
                name: name.to_string(),
                version: "".to_string(),
                exists: false,
            },
            message,
        );
    }

    let contract = load_contract_for_file(Path::new(&contract_path));
    let message = log_action(
        "contract_retrieved",
        Some("exists=true"),
        Some(&contract.contract.name),
        Some(&contract.contract.version),
        None,
    );
    (
        ContractInfo {
            name: contract.contract.name,
            version: contract.contract.version,
            exists: true,
        },
        message,
    )
}

/// Validate a contract's syntax and structure.
/// Attempts to parse the TOML file into a SchemaContracts.
/// Returns validation result and a log message.
pub fn validate_contract(name: &str) -> (ContractValidation, String) {
    let contract_path = format!("contracts/{}.toml", name);

    if !Path::new(&contract_path).exists() {
        let message = log_action(
            "contract_validated",
            Some("error=Contract not found"),
            Some(name),
            None,
            None,
        );
        return (
            ContractValidation {
                valid: false,
                error: Some("Contract not found".to_string()),
            },
            message,
        );
    }

    match std::panic::catch_unwind(|| load_contract_for_file(Path::new(&contract_path))) {
        Ok(contract) => {
            let message = log_action(
                "contract_validated",
                Some("valid=true"),
                Some(&contract.contract.name),
                Some(&contract.contract.version),
                None,
            );
            (
                ContractValidation {
                    valid: true,
                    error: None,
                },
                message,
            )
        }
        Err(_) => {
            let message = log_action(
                "contract_validated",
                Some("error=Contract failed to parse"),
                Some(name),
                None,
                None,
            );
            (
                ContractValidation {
                    valid: false,
                    error: Some("Contract failed to parse".to_string()),
                },
                message,
            )
        }
    }
}
