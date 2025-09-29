use crate::core::orchestration::run_contract_validation;
use crate::logging::schema::{AuditLogEntry, Contract, Executor};
use crate::logging::writer::log_and_print;
use chrono::Utc;
use glob::glob;
use hostname;
use std::path::Path;
use whoami;

pub async fn run_all() {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    for entry in glob("contracts/*.toml").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let contract_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                if let Err(_) = run_contract_validation(contract_name, &executor, true).await {
                    eprintln!(
                        "❌ Validation failed for {}. Check logs for details.",
                        contract_name
                    );
                }
            }
            Err(_) => eprintln!("❌ Error reading contract files. Check logs for details."),
        }
    }
}

pub async fn run_single(contract_name: &str) {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    if !Path::new(&format!("contracts/{}.toml", contract_name)).exists() {
        eprintln!(
            "❌ Contract '{}' not found. Use 'pipa contract list' to see available contracts.",
            contract_name
        );
        return;
    }

    match run_contract_validation(contract_name, &executor, true).await {
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
                executor,
                details: None,
                summary: None,
            };
            log_and_print(
                &entry,
                &format!("✅ Process complete for {}", contract_name),
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