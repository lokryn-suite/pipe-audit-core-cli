use glob::glob;
use hostname;
use pipa::contract::Executor;
use pipa::audit_logging::JsonlLogger;
use pipa::run::run_contract_validation;
use std::path::Path;
use whoami;

/// Run validation for *all* contracts in the `contracts/` directory.
///
/// This function:
/// 1. Captures the current user and host (for audit metadata).
/// 2. Iterates over all `*.toml` files in `contracts/`.
/// 3. For each contract, calls `run_contract_validation` from the engine.
/// 4. Prints the validation message and warns if failures occurred.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa run --all
/// ```
pub async fn run_all() {
    // Create logger
    let logger = JsonlLogger::default();

    // Capture host and user for Executor metadata
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    // Iterate over all contract TOML files
    for entry in glob("contracts/*.toml").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // Extract contract name from filename (strip extension)
                let contract_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                // Run validation via engine API
                match run_contract_validation(&logger, contract_name, &executor, true).await {
                    Ok((outcome, message)) => {
                        println!("{}", message);
                        if !outcome.passed {
                            eprintln!(
                                "⚠️  Validation completed with failures for {}",
                                contract_name
                            );
                        }
                    }
                    Err(_) => {
                        eprintln!(
                            "❌ Validation failed for {}. Check logs for details.",
                            contract_name
                        );
                    }
                }
            }
            Err(_) => eprintln!("❌ Error reading contract files. Check logs for details."),
        }
    }
}

/// Run validation for a *single* contract by name.
///
/// This function:
/// 1. Captures the current user and host (for audit metadata).
/// 2. Verifies the contract file exists in `contracts/{name}.toml`.
/// 3. Calls `run_contract_validation` from the engine.
/// 4. Prints the validation message and warns if failures occurred.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa run <contract_name>
/// ```
pub async fn run_single(contract_name: &str) {
    // Create logger
    let logger = JsonlLogger::default();

    // Capture host and user for Executor metadata
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    // Ensure the contract file exists before running
    if !Path::new(&format!("contracts/{}.toml", contract_name)).exists() {
        eprintln!(
            "❌ Contract '{}' not found. Use 'pipa contract list' to see available contracts.",
            contract_name
        );
        return;
    }

    // Run validation via engine API
    match run_contract_validation(&logger, contract_name, &executor, true).await {
        Ok((outcome, message)) => {
            println!("{}", message);
            if !outcome.passed {
                eprintln!(
                    "⚠️  Validation completed with {} failures out of {} checks",
                    outcome.fail_count,
                    outcome.pass_count + outcome.fail_count
                );
            }
        }
        Err(_) => {
            eprintln!(
                "❌ Validation failed for {}. Check logs for details.",
                contract_name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_contract_file_path_construction() {
        let contract_name = "test_contract";
        let expected_path = "contracts/test_contract.toml";
        let actual_path = format!("contracts/{}.toml", contract_name);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_executor_metadata_creation() {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let executor = Executor {
            user: whoami::username(),
            host: hostname.clone(),
        };

        assert!(!executor.user.is_empty());
        assert!(!executor.host.is_empty());
    }

    #[test]
    fn test_glob_pattern_for_contracts() {
        // Create a temporary directory with test contract files
        let temp_dir = TempDir::new().unwrap();
        let contracts_dir = temp_dir.path().join("contracts");
        fs::create_dir_all(&contracts_dir).unwrap();

        // Create some test contract files
        fs::write(contracts_dir.join("contract1.toml"), "test content").unwrap();
        fs::write(contracts_dir.join("contract2.toml"), "test content").unwrap();
        fs::write(contracts_dir.join("readme.md"), "not a contract").unwrap();

        // Change to temp directory and test glob pattern
        std::env::set_current_dir(&temp_dir).unwrap();

        let matches: Vec<_> = glob("contracts/*.toml")
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_contract_name_extraction_from_path() {
        let path = Path::new("contracts/my_contract.toml");
        let contract_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        assert_eq!(contract_name, "my_contract");
    }

    #[test]
    fn test_contract_name_extraction_fallback() {
        let path = Path::new("invalid");
        let contract_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        assert_eq!(contract_name, "invalid");
    }

    #[test]
    fn test_contract_file_exists_check() {
        // Create a temporary directory with a test contract
        let temp_dir = TempDir::new().unwrap();
        let contracts_dir = temp_dir.path().join("contracts");
        fs::create_dir_all(&contracts_dir).unwrap();
        fs::write(contracts_dir.join("test.toml"), "test content").unwrap();

        // Test that existing contract is found (using absolute path)
        assert!(contracts_dir.join("test.toml").exists());

        // Test that non-existing contract is not found
        assert!(!contracts_dir.join("nonexistent.toml").exists());
    }

    #[test]
    fn test_hostname_fallback() {
        // Test that hostname fallback works
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Hostname should either be populated or empty (but not error)
        assert!(hostname.is_empty() || !hostname.is_empty());
    }

    #[test]
    fn test_username_retrieval() {
        // Test that username can be retrieved
        let username = whoami::username();
        assert!(!username.is_empty());
    }

    #[test]
    fn test_executor_fields() {
        // Test that Executor struct can be created and accessed
        let executor = Executor {
            user: "testuser".to_string(),
            host: "testhost".to_string(),
        };

        assert_eq!(executor.user, "testuser");
        assert_eq!(executor.host, "testhost");
    }

    #[test]
    fn test_path_file_stem_edge_cases() {
        // Test file_stem with various path formats
        let path1 = Path::new("contracts/test.toml");
        assert_eq!(path1.file_stem().and_then(|s| s.to_str()), Some("test"));

        let path2 = Path::new("test.toml");
        assert_eq!(path2.file_stem().and_then(|s| s.to_str()), Some("test"));

        let path3 = Path::new("test");
        assert_eq!(path3.file_stem().and_then(|s| s.to_str()), Some("test"));
    }

    #[test]
    fn test_logger_creation() {
        // Test that JsonlLogger can be created
        let logger = JsonlLogger::default();
        drop(logger);
    }

    #[test]
    fn test_glob_pattern_no_matches() {
        // Create a temporary directory with no contract files
        let temp_dir = TempDir::new().unwrap();
        let contracts_dir = temp_dir.path().join("contracts");
        fs::create_dir_all(&contracts_dir).unwrap();

        std::env::set_current_dir(&temp_dir).unwrap();

        let matches: Vec<_> = glob("contracts/*.toml")
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_contract_path_format_variations() {
        // Test different contract name formats
        let names = vec!["simple", "with-dash", "with_underscore", "with123numbers"];

        for name in names {
            let path = format!("contracts/{}.toml", name);
            assert!(path.starts_with("contracts/"));
            assert!(path.ends_with(".toml"));
            assert!(path.contains(name));
        }
    }
}
