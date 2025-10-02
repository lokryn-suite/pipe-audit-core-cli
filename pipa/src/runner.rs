//! CLI-facing validation runner.
//!
//! This module provides a high-level entry point (`validate_data`) that
//! orchestrates contract validation, profile loading, and file movement.
//! It is intended for use by the CLI or other top-level consumers.
//!
//! Responsibilities:
//! - Construct an `Executor` (user + host metadata).
//! - Run contract validation via the engine.
//! - Decide pass/fail outcome.
//! - Load profiles for file movement.
//! - Move data to destination or quarantine as appropriate.
//! - Emit audit log entries for errors and movement events.

use crate::contracts::SchemaContracts;
use crate::engine::contracts::run_contract_validation;
use crate::logging::schema::Executor;
use crate::movement::FileMovement;
use crate::profiles::load_profiles;
use crate::logging::AuditLogEntry;
use crate::logging::writer::log_and_print;
use anyhow::Result;
use hostname;
use whoami;
use chrono::Utc;

/// Validate a dataset against a contract, then move it based on outcome.
///
/// # Arguments
/// * `data` - Raw file contents (CSV, Parquet, etc.).
/// * `extension` - File extension (used to select driver).
/// * `contracts` - Parsed schema contract.
///
/// # Returns
/// * `Result<()>` - Ok if orchestration succeeded, Err if a fatal error occurred.
///
/// # Behavior
/// - Runs validation via `run_contract_validation`.
/// - If validation passes and a destination is configured, writes data to destination.
/// - If validation fails and a quarantine is configured, writes data to quarantine.
/// - Logs all major events to the audit log and optionally to console.
pub async fn validate_data(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
) -> Result<()> {
    // --- Build executor metadata (user + host) ---
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    // --- Run validation ---
    let (outcome, _log) = run_contract_validation(
        &contracts.contract.name,
        &executor,
        true, // log_to_console = true for CLI
    )
    .await?;

    let validation_passed = outcome.fail_count == 0;

    // --- Load profiles for movement ---
    let profiles = match load_profiles() {
        Ok(profiles) => profiles,
        Err(_) => {
            log_and_print(
                &AuditLogEntry {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "AUDIT",
                    event: "error",
                    contract: None,
                    target: None,
                    results: None,
                    executor: executor.clone(),
                    details: Some("Failed to load profiles for file movement"),
                    summary: None,
                },
                "❌ Failed to load profiles for file movement",
            );
            return Ok(()); // gracefully exit without movement
        }
    };

    // --- Determine original location ---
    let original_location = contracts
        .source
        .as_ref()
        .and_then(|s| s.location.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    // --- Parse DataFrame for movement ---
    let driver = crate::drivers::get_driver(extension)?;
    let df = driver.load(data)?;

    // --- Check movement configuration ---
    let dest_valid = contracts
        .destination
        .as_ref()
        .map(|d| d.r#type.as_str() != "not_moved")
        .unwrap_or(false);

    let quarantine_valid = contracts
        .quarantine
        .as_ref()
        .map(|q| q.r#type.as_str() != "not_moved")
        .unwrap_or(false);

    // --- Movement logic ---
    if validation_passed && dest_valid {
        if let Some(destination) = &contracts.destination {
            FileMovement::write_success_data(&df, original_location, destination, &profiles).await?;
            // TODO: log success event
        }
    } else if !validation_passed && quarantine_valid {
        if let Some(quarantine) = &contracts.quarantine {
            FileMovement::write_quarantine_data(&df, original_location, quarantine, &profiles).await?;
            // TODO: log quarantine event
        }
    }

    Ok(())
}
