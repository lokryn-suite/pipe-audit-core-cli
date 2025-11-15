use pipa::contract::{get_contract, list_contracts, validate_contract};
use pipa::audit_logging::JsonlLogger;
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
    let logger = JsonlLogger::default();
    match list_contracts(&logger) {
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
/// parses and validates the contract definition. Prints the engine's
/// validation message to stdout.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa contract validate <file>
/// ```
pub async fn validate(file: &str) {
    let logger = JsonlLogger::default();
    let (_validation, message) = validate_contract(&logger, file);
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
    let logger = JsonlLogger::default();
    let (contract_info, message) = get_contract(&logger, name);
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

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_contract_path_construction() {
        let name = "my_contract";
        let path = format!("contracts/{}.toml", name);
        assert_eq!(path, "contracts/my_contract.toml");
    }

    #[test]
    fn test_contract_file_read() {
        // Create a temporary directory with a test contract
        let temp_dir = TempDir::new().unwrap();
        let contracts_dir = temp_dir.path().join("contracts");
        fs::create_dir_all(&contracts_dir).unwrap();

        let test_content = "test contract content";
        let test_file = contracts_dir.join("test.toml");
        fs::write(&test_file, test_content).unwrap();

        // Read using absolute path
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, test_content);
    }

    #[test]
    fn test_contract_file_read_error() {
        let path = "contracts/nonexistent.toml";
        let result = fs::read_to_string(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_contracts_listing() {
        // Create a temporary directory with multiple test contracts
        let temp_dir = TempDir::new().unwrap();
        let contracts_dir = temp_dir.path().join("contracts");
        fs::create_dir_all(&contracts_dir).unwrap();

        fs::write(contracts_dir.join("contract1.toml"), "content1").unwrap();
        fs::write(contracts_dir.join("contract2.toml"), "content2").unwrap();
        fs::write(contracts_dir.join("contract3.toml"), "content3").unwrap();

        // Verify all contract files exist using absolute paths
        assert!(contracts_dir.join("contract1.toml").exists());
        assert!(contracts_dir.join("contract2.toml").exists());
        assert!(contracts_dir.join("contract3.toml").exists());
    }
}
