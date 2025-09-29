use crate::contracts::SchemaContracts;
use crate::core::validation::execute_validation;
use crate::logging::schema::{AuditLogEntry, Contract, Executor};
use crate::logging::writer::log_and_print;
use anyhow::Result;
use chrono::Utc;
use hostname;
use whoami;

#[cfg(feature = "file-management")]
use crate::movement::FileMovement;
#[cfg(feature = "file-management")]
use crate::profiles::load_profiles;

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

    // Call core validation (handles all validation audit logging internally)
    let results = execute_validation(data, extension, contracts, &executor).await?;

    // Count results
    let pass_count = results.iter().filter(|r| r.result == "pass").count();
    let fail_count = results.iter().filter(|r| r.result == "fail").count();

    // Final completion event
    log_and_print(
        &AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "validation_complete",
            contract: Some(Contract {
                name: &contracts.contract.name,
                version: &contracts.contract.version,
            }),
            target: None,
            results: Some(results.clone()),
            executor: executor.clone(),
            details: None,
            summary: None,
        },
        &format!(
            "✅ Contract {} v{}: {} PASS, {} FAIL",
            contracts.contract.name, contracts.contract.version, pass_count, fail_count
        ),
    );

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

        let dest_valid = contracts.destination.as_ref()
            .and_then(|d| d.r#type.as_ref())
            .map(|t| t != "not_moved")
            .unwrap_or(false);
            
        let quarantine_valid = contracts.quarantine.as_ref()
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