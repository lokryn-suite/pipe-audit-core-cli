use crate::contracts::SchemaContracts;
use crate::core::orchestration::run_contract_validation;
use crate::logging::schema::Executor;
use anyhow::Result;
use hostname;
use whoami;

#[cfg(feature = "file-management")]
use crate::movement::FileMovement;
#[cfg(feature = "file-management")]
use crate::profiles::load_profiles;

pub async fn validate_data(
    _data: &[u8],
    _extension: &str,
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

    // Use orchestration layer with console output
    let _outcome = run_contract_validation(
        &contracts.contract.name,
        &executor,
        true, // log_to_console = true for CLI
    ).await?;


    // File movement logic (CLI-specific feature)
    #[cfg(feature = "file-management")]
    {
        let validation_passed = fail_count == 0;

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
            .and_then(|d| d.r#type.as_ref())
            .map(|t| t != "not_moved")
            .unwrap_or(false);

        let quarantine_valid = contracts
            .quarantine
            .as_ref()
            .and_then(|q| q.r#type.as_ref())
            .map(|t| t != "not_moved")
            .unwrap_or(false);

        if validation_passed && dest_valid {
            if let Some(destination) = &contracts.destination {
                match FileMovement::write_success_data(
                    &df,
                    original_location,
                    destination,
                    &profiles,
                )
                .await
                {
                    Ok(_) => log_and_print(
                        &AuditLogEntry {
                            timestamp: Utc::now().to_rfc3339(),
                            level: "AUDIT",
                            event: "file_written",
                            contract: None,
                            target: None,
                            results: None,
                            executor: executor.clone(),
                            details: Some("success data written"),
                            summary: None,
                        },
                        "✅ Data written to destination",
                    ),
                    Err(e) => log_and_print(
                        &AuditLogEntry {
                            timestamp: Utc::now().to_rfc3339(),
                            level: "AUDIT",
                            event: "error",
                            contract: None,
                            target: None,
                            results: None,
                            executor: executor.clone(),
                            details: Some(&format!("Failed to write to destination: {}", e)),
                            summary: None,
                        },
                        &format!("❌ Failed to write to destination: {}", e),
                    ),
                }
            }
        } else if !validation_passed && quarantine_valid {
            if let Some(quarantine) = &contracts.quarantine {
                match FileMovement::write_quarantine_data(
                    &df,
                    original_location,
                    quarantine,
                    &profiles,
                )
                .await
                {
                    Ok(_) => log_and_print(
                        &AuditLogEntry {
                            timestamp: Utc::now().to_rfc3339(),
                            level: "AUDIT",
                            event: "file_written",
                            contract: None,
                            target: None,
                            results: None,
                            executor: executor.clone(),
                            details: Some("data quarantined"),
                            summary: None,
                        },
                        "⚠️ Data quarantined",
                    ),
                    Err(e) => log_and_print(
                        &AuditLogEntry {
                            timestamp: Utc::now().to_rfc3339(),
                            level: "AUDIT",
                            event: "error",
                            contract: None,
                            target: None,
                            results: None,
                            executor: executor.clone(),
                            details: Some(&format!("Failed to write to quarantine: {}", e)),
                            summary: None,
                        },
                        &format!("❌ Failed to write to quarantine: {}", e),
                    ),
                }
            }
        }
    }

    Ok(())
}
