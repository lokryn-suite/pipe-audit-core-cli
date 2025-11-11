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
