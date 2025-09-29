//! Orchestration layer that coordinates business logic with logging
//! Both CLI and API call these functions

use crate::connectors::{AzureConnector, Connector, GCSConnector, S3Connector};
use crate::core::validation::execute_validation;
use crate::error::ValidationResult;
use crate::logging::schema::{AuditLogEntry, Contract, Executor, RuleResult, Target};
use crate::logging::writer::{log_and_print, log_event};
use crate::profiles::{load_profiles, Profiles};
use chrono::Utc;
use std::path::Path as StdPath;

/// Result of running a contract validation
pub struct ValidationOutcome {
    pub passed: bool,
    pub pass_count: usize,
    pub fail_count: usize,
    pub results: Vec<RuleResult>,
}

#[derive(Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub contracts_dir_exists: bool,
    pub logs_dir_exists: bool,
    pub profile_count: usize,
}

/// Helper to log with optional console output
fn log_audit(entry: &AuditLogEntry, console_msg: &str, log_to_console: bool) {
    if log_to_console {
        log_and_print(entry, console_msg);
    } else {
        log_event(entry);
    }
}

/// Run a single contract validation
/// Handles: file acquisition, logging, validation execution
pub async fn run_contract_validation(
    contract_name: &str,
    executor: &Executor,
    log_to_console: bool,
) -> ValidationResult<ValidationOutcome> {
    let contract_path = format!("contracts/{}.toml", contract_name);

    if !StdPath::new(&contract_path).exists() {
        return Err(crate::error::ValidationError::Other(format!(
            "Contract '{}' not found",
            contract_name
        )));
    }

    let contracts = crate::contracts::load_contract_for_file(StdPath::new(&contract_path));
    let profiles = load_profiles()?;

    let source = contracts.source.as_ref().ok_or_else(|| {
        crate::error::ValidationError::Other("Contract missing source".to_string())
    })?;

    let location = source.location.as_ref().ok_or_else(|| {
        crate::error::ValidationError::Other("Source missing location".to_string())
    })?;

    // Log file acquisition
    log_audit(
        &AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "file_acquired",
            contract: None,
            target: Some(Target {
                file: location,
                column: None,
                rule: None,
            }),
            results: None,
            executor: executor.clone(),
            details: source
                .profile
                .as_ref()
                .map(|p| format!("profile={}", p))
                .as_deref(),
            summary: None,
        },
        &format!(
            "🔎 Fetching {} via profile {}",
            location,
            source.profile.as_ref().unwrap_or(&"local".to_string())
        ),
        log_to_console,
    );

    // Fetch data
    let data = fetch_data_from_source(source, &profiles).await?;

    // Log file read
    log_audit(
        &AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "file_read",
            contract: None,
            target: Some(Target {
                file: location,
                column: None,
                rule: None,
            }),
            results: None,
            executor: executor.clone(),
            details: Some(&format!("bytes={}", data.len())),
            summary: None,
        },
        &format!("📊 Read {} bytes", data.len()),
        log_to_console,
    );

    // Get extension
    let extension = source
        .location
        .as_ref()
        .and_then(|loc| StdPath::new(loc).extension().and_then(|s| s.to_str()))
        .unwrap_or("csv");

    // Run validation (logs internally)
    let results = execute_validation(&data, extension, &contracts, executor).await?;

    // Count results
    let pass_count = results.iter().filter(|r| r.result == "pass").count();
    let fail_count = results.iter().filter(|r| r.result == "fail").count();

    // Log completion
    log_audit(
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
        log_to_console,
    );

    Ok(ValidationOutcome {
        passed: fail_count == 0,
        pass_count,
        fail_count,
        results,
    })
}

/// Fetch data from a source configuration
async fn fetch_data_from_source(
    source: &crate::contracts::schema::Source,
    profiles: &Profiles,
) -> ValidationResult<Vec<u8>> {
    let location = source.location.as_ref().ok_or_else(|| {
        crate::error::ValidationError::Other("Source missing location".to_string())
    })?;

    match source.r#type.as_str() {
        "local" => std::fs::read(location).map_err(|e| crate::error::ValidationError::Io(e)),
        "s3" => {
            let profile_name = source.profile.as_ref().ok_or_else(|| {
                crate::error::ValidationError::Other("S3 source requires profile".to_string())
            })?;
            let profile = profiles.get(profile_name).ok_or_else(|| {
                crate::error::ValidationError::ProfileNotFound(profile_name.clone())
            })?;

            let url = url::Url::parse(location)
                .map_err(|_| crate::error::ValidationError::Other("Invalid URL".to_string()))?;

            let connector = S3Connector::from_profile_and_url(profile, &url)
                .await
                .map_err(|e| crate::error::ValidationError::Connector(e.to_string()))?;

            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| crate::error::ValidationError::Connector(e.to_string()))?;

            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)
                .map_err(|e| crate::error::ValidationError::Io(e))?;

            Ok(buffer)
        }
        "azure" => {
            let profile_name = source.profile.as_ref().ok_or_else(|| {
                crate::error::ValidationError::Other("Azure source requires profile".to_string())
            })?;
            let profile = profiles.get(profile_name).ok_or_else(|| {
                crate::error::ValidationError::ProfileNotFound(profile_name.clone())
            })?;

            let url = url::Url::parse(location)
                .map_err(|_| crate::error::ValidationError::Other("Invalid URL".to_string()))?;

            let connector = AzureConnector::from_profile_and_url(profile, &url)
                .await
                .map_err(|e| crate::error::ValidationError::Connector(e.to_string()))?;

            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| crate::error::ValidationError::Connector(e.to_string()))?;

            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)
                .map_err(|e| crate::error::ValidationError::Io(e))?;

            Ok(buffer)
        }
        "gcs" => {
            let profile_name = source.profile.as_ref().ok_or_else(|| {
                crate::error::ValidationError::Other("GCS source requires profile".to_string())
            })?;
            let profile = profiles.get(profile_name).ok_or_else(|| {
                crate::error::ValidationError::ProfileNotFound(profile_name.clone())
            })?;

            let url = url::Url::parse(location)
                .map_err(|_| crate::error::ValidationError::Other("Invalid URL".to_string()))?;

            let connector = GCSConnector::from_profile_and_url(profile, &url)
                .await
                .map_err(|e| crate::error::ValidationError::Connector(e.to_string()))?;

            let mut reader = connector
                .fetch(location)
                .await
                .map_err(|e| crate::error::ValidationError::Connector(e.to_string()))?;

            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)
                .map_err(|e| crate::error::ValidationError::Io(e))?;

            Ok(buffer)
        }
        _ => Err(crate::error::ValidationError::Other(format!(
            "Unsupported source type: {}",
            source.r#type
        ))),
    }
}

pub fn check_system_health() -> HealthStatus {
    let contracts_exist = StdPath::new("contracts").exists();
    let logs_exist = StdPath::new("logs").exists();
    let profile_count = load_profiles().map(|p| p.len()).unwrap_or(0);

    HealthStatus {
        healthy: contracts_exist && logs_exist && profile_count > 0,
        contracts_dir_exists: contracts_exist,
        logs_dir_exists: logs_exist,
        profile_count,
    }
}

/// Run health check with logging
pub fn run_health_check(executor: &Executor, log_to_console: bool) -> HealthStatus {
    let status = check_system_health();

    let log_fn = |entry: &AuditLogEntry, msg: &str| {
        if log_to_console {
            log_and_print(entry, msg);
        } else {
            log_event(entry);
        }
    };

    if status.contracts_dir_exists {
        log_fn(
            &AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "health_check",
                contract: None,
                target: None,
                results: None,
                executor: executor.clone(),
                details: Some("contracts directory exists"),
                summary: None,
            },
            "✅ contracts directory exists",
        );
    }

    if status.logs_dir_exists {
        log_fn(
            &AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "health_check",
                contract: None,
                target: None,
                results: None,
                executor: executor.clone(),
                details: Some("logs directory exists"),
                summary: None,
            },
            "✅ logs directory exists",
        );
    }

    log_fn(
        &AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_check",
            contract: None,
            target: None,
            results: None,
            executor: executor.clone(),
            details: Some(&format!("{} profiles loaded", status.profile_count)),
            summary: None,
        },
        &format!("✅ {} profiles loaded", status.profile_count),
    );

    let summary_msg = if status.healthy {
        "system healthy"
    } else {
        "system unhealthy"
    };

    log_fn(
        &AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_summary",
            contract: None,
            target: None,
            results: None,
            executor: executor.clone(),
            details: Some(summary_msg),
            summary: None,
        },
        &format!("📊 System status: {}", summary_msg),
    );

    status
}