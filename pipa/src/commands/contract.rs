use pipa::contract::{get_contract, list_contracts, validate_contract};
use std::fs;

/// List all available contracts in the project.
///
/// Delegates to `pipa::contract::list_contracts()`, which scans
/// the `contracts/` directory and returns a list of contract names.
/// Prints a summary message and each contract name to stdout.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa contract list
/// ```
pub async fn list() {
    match list_contracts() {
        Ok((contract_list, message)) => {
            // Print engine-provided summary message
            println!("{}", message);

            // Print each contract name in the list
            for name in contract_list.contracts {
                println!("  - {}", name);
            }
        }
        Err(_) => {
            eprintln!("❌ Failed to read contracts directory. Check logs for details.");
        }
    }
}

/// Validate a contract file for TOML syntax and schema correctness.
///
/// Delegates to `pipa::contract::validate_contract(file)`, which
/// parses and validates the contract definition. Prints the engine’s
/// validation message to stdout.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa contract validate <file>
/// ```
pub async fn validate(file: &str) {
    let (_validation, message) = validate_contract(file);
    println!("{}", message);
}

/// Show details of a specific contract by name.
///
/// Delegates to `pipa::contract::get_contract(name)`, which returns
/// metadata about the contract. If the contract exists, this function
/// also reads and prints the raw TOML file contents for inspection.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa contract show <name>
/// ```
pub async fn show(name: &str) {
    let (contract_info, message) = get_contract(name);
    println!("{}", message);

    if contract_info.exists {
        let path = format!("contracts/{}.toml", name);
        match fs::read_to_string(&path) {
            Ok(content) => {
                println!("📄 Contract: {}", name);
                println!("\n{}", content);
            }
            Err(_) => eprintln!("❌ Failed to read contract content"),
        }
    }
}
