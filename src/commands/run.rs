use crate::connectors::{AzureConnector, Connector, GCSConnector, S3Connector};
use crate::contracts::load_contract_for_file;
use crate::logging::schema::{AuditLogEntry, Contract, Executor, Target};
use crate::logging::writer::log_and_print;
use crate::profiles::load_profiles;
use crate::runner;

use chrono::Utc;
use glob::glob;
use hostname;
use std::path::Path;
use whoami;

pub async fn run_all() {
    let profiles = match load_profiles() {
        Ok(profiles) => profiles,
        Err(_) => {
            eprintln!("❌ Validation failed. Check logs for details.");
            return;
        }
    };

    for entry in glob("contracts/*.toml").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let contract_file = path.to_string_lossy().to_string();
                if let Err(_) = validate_with_contract(&contract_file, &profiles).await {
                    eprintln!(
                        "❌ Validation failed for {}. Check logs for details.",
                        path.file_stem().unwrap_or_default().to_string_lossy()
                    );
                }
            }
            Err(_) => eprintln!("❌ Error reading contract files. Check logs for details."),
        }
    }
}

pub async fn run_single(contract_name: &str) {
    let profiles = match load_profiles() {
        Ok(profiles) => profiles,
        Err(_) => {
            eprintln!("❌ Validation failed. Check logs for details.");
            return;
        }
    };

    let contract_file = format!("contracts/{}.toml", contract_name);

    if !Path::new(&contract_file).exists() {
        eprintln!(
            "❌ Contract '{}' not found. Use 'pipa contract list' to see available contracts.",
            contract_name
        );
        return;
    }

    match validate_with_contract(&contract_file, &profiles).await {
        Ok(_) => {
            let entry = AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "process_complete",
                contract: Some(Contract {
                    name: contract_name,
                    version: "0.1.0",
                }),
                target: None,
                results: None,
                executor: Executor {
                    user: whoami::username(),
                    host: hostname::get()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                },
                details: None,
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("✅ Validation passed for {}", contract_name),
            );
        }
        Err(_) => {
            eprintln!(
                "❌ Validation failed for {}. Check logs for details.",
                contract_name
            );
        }
    }
}

async fn validate_with_contract(
    contract_path: &str,
    profiles: &crate::profiles::Profiles,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(contract_path);
    let contracts = load_contract_for_file(path);

    let source = contracts
        .source
        .as_ref()
        .ok_or("Contract missing [source] section")?;

    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    let data: Vec<u8> = match source.r#type.as_str() {
        "local" => {
            let location = source
                .location
                .as_ref()
                .ok_or("Local source missing location")?;

            let entry = AuditLogEntry {
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
                details: None,
                summary: None,
            };
            log_and_print(&entry, &format!("📂 Reading local file {}", location));

            let mut file = std::fs::File::open(location)?;
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buffer)?;

            let entry = AuditLogEntry {
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
                details: Some(&format!("bytes={}", buffer.len())),
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("📊 Read {} bytes from local file", buffer.len()),
            );

            buffer
        }
        "s3" => {
            let profile_name = source
                .profile
                .as_ref()
                .ok_or("S3 source requires profile")?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
            let location = source
                .location
                .as_ref()
                .ok_or("S3 source missing location")?;

            // Audit + console: starting fetch
            let entry = AuditLogEntry {
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
                details: Some(&format!("profile={}", profile_name)),
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("🔎 Fetching {} via profile {}", location, profile_name),
            );

            let url = url::Url::parse(location)?;
            let connector = S3Connector::from_profile_and_url(profile, &url).await?;
            let mut reader = connector.fetch(location).await?;

            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)?;

            // Audit + console: file read
            let entry = AuditLogEntry {
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
                details: Some(&format!("bytes={}", buffer.len())),
                summary: None,
            };
            log_and_print(&entry, &format!("📊 Read {} bytes from S3", buffer.len()));

            buffer
        }

        "azure" => {
            let profile_name = source
                .profile
                .as_ref()
                .ok_or("Azure source requires profile")?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
            let location = source
                .location
                .as_ref()
                .ok_or("Azure source missing location")?;

            // Audit + console: starting fetch
            let entry = AuditLogEntry {
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
                details: Some(&format!("profile={}", profile_name)),
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("☁️ Fetching {} via profile {}", location, profile_name),
            );

            let url = url::Url::parse(location)?;
            let connector = AzureConnector::from_profile_and_url(profile, &url).await?;

            // Attempt fetch
            let mut reader = match connector.fetch(location).await {
                Ok(r) => r,
                Err(e) => {
                    let entry = AuditLogEntry {
                        timestamp: Utc::now().to_rfc3339(),
                        level: "AUDIT",
                        event: "error",
                        contract: None,
                        target: Some(Target {
                            file: location,
                            column: None,
                            rule: None,
                        }),
                        results: None,
                        executor,
                        details: Some("Azure fetch failed"),
                        summary: None,
                    };
                    log_and_print(&entry, &format!("❌ Azure fetch failed for {}", location));
                    return Err(e.into());
                }
            };

            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)?;

            // Audit + console: file read
            let entry = AuditLogEntry {
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
                details: Some(&format!("bytes={}", buffer.len())),
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("📊 Read {} bytes from Azure", buffer.len()),
            );

            buffer
        }

        "gcs" => {
            let profile_name = source
                .profile
                .as_ref()
                .ok_or("GCS source requires profile")?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
            let location = source
                .location
                .as_ref()
                .ok_or("GCS source missing location")?;

            // Audit + console: starting fetch
            let entry = AuditLogEntry {
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
                details: Some(&format!("profile={}", profile_name)),
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("🔎 Fetching {} via profile {}", location, profile_name),
            );

            let url = url::Url::parse(location)?;
            let connector = GCSConnector::from_profile_and_url(profile, &url).await?;

            // Attempt fetch
            let mut reader = match connector.fetch(location).await {
                Ok(r) => r,
                Err(e) => {
                    let entry = AuditLogEntry {
                        timestamp: Utc::now().to_rfc3339(),
                        level: "AUDIT",
                        event: "error",
                        contract: None,
                        target: Some(Target {
                            file: location,
                            column: None,
                            rule: None,
                        }),
                        results: None,
                        executor,
                        details: Some("GCS fetch failed"),
                        summary: None,
                    };
                    log_and_print(&entry, &format!("❌ GCS fetch failed for {}", location));
                    return Err(e.into());
                }
            };

            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)?;

            // Audit + console: file read
            let entry = AuditLogEntry {
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
                details: Some(&format!("bytes={}", buffer.len())),
                summary: None,
            };
            log_and_print(&entry, &format!("📊 Read {} bytes from GCS", buffer.len()));

            buffer
        }

        "sftp" => {
            let location = source
                .location
                .as_ref()
                .ok_or("SFTP source missing location")?;
            let entry = AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "unsupported_source",
                contract: None,
                target: Some(Target {
                    file: location,
                    column: None,
                    rule: None,
                }),
                results: None,
                executor,
                details: Some("SFTP connector not implemented"),
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("🔐 SFTP fetch not yet implemented for {}", location),
            );
            return Err("SFTP connector not implemented".into());
        }
        "not_moved" => {
            let entry = AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "skipped_source",
                contract: None,
                target: None,
                results: None,
                executor: executor.clone(),
                details: Some("Source marked as not_moved"),
                summary: None,
            };
            log_and_print(&entry, "⚠️ Source marked as not_moved, skipping");
            return Ok(());
        }
        other => return Err(format!("Unsupported source type: {}", other).into()),
    };

    let extension = source
        .location
        .as_ref()
        .and_then(|loc| Path::new(loc).extension().and_then(|s| s.to_str()))
        .unwrap_or("csv");

    runner::validate_data(&data, extension, &contracts).await?;

    Ok(())
}
