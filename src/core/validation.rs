use crate::contracts::SchemaContracts;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;
use crate::error::ValidationResult;
use crate::logging::schema::{AuditLogEntry, Contract, Executor, RuleResult};
use crate::logging::writer::log_event;
use anyhow::Context;
use chrono::Utc;

/// Core validation orchestration - audit logging only, no console output
pub async fn execute_validation(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
    executor: &Executor,
) -> ValidationResult<Vec<RuleResult>> {
    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "validation_start",
        contract: Some(Contract {
            name: &contracts.contract.name,
            version: &contracts.contract.version,
        }),
        target: None,
        results: None,
        executor: executor.clone(),
        details: Some(&format!("bytes={}, extension={}", data.len(), extension)),
        summary: None,
    });

    let driver =
        get_driver(extension).context("Failed to find a suitable driver for the extension")?;

    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "driver_found",
        contract: None,
        target: None,
        results: None,
        executor: executor.clone(),
        details: Some(&format!("extension={}", extension)),
        summary: None,
    });

    let df = driver
        .load(data)
        .context("Failed to parse data from memory")?;

    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "dataframe_parsed",
        contract: None,
        target: None,
        results: None,
        executor: executor.clone(),
        details: Some(&format!("rows={}, cols={}", df.height(), df.width())),
        summary: None,
    });

    let results: Vec<RuleResult> = validate_dataframe(&df, contracts)?;

    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "validation_summary",
        contract: Some(Contract {
            name: &contracts.contract.name,
            version: &contracts.contract.version,
        }),
        target: None,
        results: Some(results.clone()),
        executor: executor.clone(),
        details: None,
        summary: None,
    });

    Ok(results)
}
