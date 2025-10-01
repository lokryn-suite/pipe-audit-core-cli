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


pub async fn validate_data(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
) -> Result<()> {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    // Run validation
    let (outcome, _log)= run_contract_validation(
        &contracts.contract.name,
        &executor,
        true, // log_to_console = true for CLI
    )
    .await?;

    // Decide pass/fail
    let validation_passed = outcome.fail_count == 0;

    // Load profiles for movement
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
            return Ok(());
        }
    };

    let original_location = contracts
        .source
        .as_ref()
        .and_then(|s| s.location.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    // Parse DataFrame for file movement
    let driver = crate::drivers::get_driver(extension)?;
    let df = driver.load(data)?;

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

    if validation_passed && dest_valid {
        if let Some(destination) = &contracts.destination {
            FileMovement::write_success_data(&df, original_location, destination, &profiles).await?;
            // log success…
        }
    } else if !validation_passed && quarantine_valid {
        if let Some(quarantine) = &contracts.quarantine {
            FileMovement::write_quarantine_data(&df, original_location, quarantine, &profiles).await?;
            // log quarantine…
        }
    }

    Ok(())
}
