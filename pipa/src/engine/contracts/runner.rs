use crate::connectors::fetch::fetch_data_from_source; // fetch raw bytes from source connector
use crate::contracts::load_contract_for_file;         // load TOML contract into SchemaContracts
use crate::engine::log_action;                        // audit logging
use crate::engine::validation::execute_validation;    // run validators against data
use crate::logging::error::{ValidationError, ValidationResult};
use crate::logging::schema::{Executor, RuleResult, Contract as LogContract};
use crate::logging::{AuditLogEntry, writer::log_and_print};
use crate::movement::FileMovement;                    // handles writing success/quarantine data
use crate::profiles::{Profiles, load_profiles};       // profile management
use chrono::Utc;
use std::path::Path as StdPath;

/// Outcome of running a contract validation
pub struct ValidationOutcome {
    pub passed: bool,           // true if no rules failed
    pub pass_count: usize,      // number of passing rules
    pub fail_count: usize,      // number of failing rules
    pub results: Vec<RuleResult>, // detailed results per rule
}

/// Run a contract validation end-to-end:
/// - Load contract + profiles
/// - Fetch data from source
/// - Execute validations
/// - Move data to destination or quarantine
/// - Log all actions
pub async fn run_contract_validation(
    contract_name: &str,
    executor: &Executor,
    log_to_console: bool,
) -> ValidationResult<(ValidationOutcome, String)> {
    // --- Contract existence check ---
    let contract_path = format!("contracts/{}.toml", contract_name);
    if !StdPath::new(&contract_path).exists() {
        return Err(ValidationError::Other(format!(
            "Contract '{}' not found",
            contract_name
        )));
    }

    // --- Load contract + profiles ---
    let contracts = load_contract_for_file(StdPath::new(&contract_path));
    let profiles: Profiles = load_profiles()?;

    // --- Validate source config ---
    let source = contracts
        .source
        .as_ref()
        .ok_or_else(|| ValidationError::Other("Contract missing source".to_string()))?;
    let location = source
        .location
        .as_ref()
        .ok_or_else(|| ValidationError::Other("Source missing location".to_string()))?;

    // --- Start log ---
    let start_message = log_action("contract_validation_started", None, Some(contract_name), None, None);
    if log_to_console {
        println!("{}", start_message);
    }

    // --- Fetch data ---
    let data = fetch_data_from_source(source, &profiles).await?;
    let _ = log_action("file_read", Some(&format!("bytes={}", data.len())), None, None, Some(location));

    // --- Determine file extension ---
    let extension = source
        .location
        .as_ref()
        .and_then(|loc| StdPath::new(loc).extension().and_then(|s| s.to_str()))
        .unwrap_or("csv");

    // --- Execute validations ---
    let results = execute_validation(&data, extension, &contracts, executor).await?;
    let pass_count = results.iter().filter(|r| r.result == "pass").count();
    let fail_count = results.iter().filter(|r| r.result == "fail").count();
    let validation_passed = fail_count == 0;

    // --- Load DataFrame for movement ---
    let original_location = source.location.as_deref().unwrap_or("unknown");
    let driver = crate::drivers::get_driver(extension)?;
    let df = driver.load(&data)?;

    // --- Movement logic ---
    if validation_passed {
        if let Some(dest) = &contracts.destination {
            if dest.r#type != "not_moved" {
                match FileMovement::write_success_data(&df, original_location, dest, &profiles).await {
                    Ok(_) => log_and_print(&AuditLogEntry { /* ... */ }, "✅ Data written to destination"),
                    Err(e) => log_and_print(&AuditLogEntry { /* ... */ }, &format!("❌ Failed to write to destination: {}", e)),
                }
            }
        }
    } else {
        if let Some(quarantine) = &contracts.quarantine {
            if quarantine.r#type != "not_moved" {
                match FileMovement::write_quarantine_data(&df, original_location, quarantine, &profiles).await {
                    Ok(_) => log_and_print(&AuditLogEntry { /* ... */ }, "⚠️ Data quarantined"),
                    Err(e) => log_and_print(&AuditLogEntry { /* ... */ }, &format!("❌ Failed to write to quarantine: {}", e)),
                }
            }
        }
    }

    // --- Completion log ---
    let details = format!("pass={}, fail={}", pass_count, fail_count);
    let message = log_action(
        "contract_validation_completed",
        Some(&details),
        Some(&contracts.contract.name),
        Some(&contracts.contract.version),
        None,
    );
    if log_to_console {
        println!("{}", message);
    }

    Ok((
        ValidationOutcome {
            passed: validation_passed,
            pass_count,
            fail_count,
            results,
        },
        message,
    ))
}
