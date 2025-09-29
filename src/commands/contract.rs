use crate::contracts::SchemaContracts;
use crate::logging::schema::{AuditLogEntry, Contract, Executor};
use crate::logging::writer::log_and_print;
use chrono::Utc;
use glob::glob;
use hostname;
use std::fs;
use whoami;

fn executor() -> Executor {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Executor {
        user: whoami::username(),
        host: hostname,
    }
}

pub async fn list() {
    let mut names = Vec::new();

    match glob("contracts/*.toml") {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_stem().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
        Err(_) => {
            eprintln!("❌ Failed to read contracts directory. Check logs for details.");
            return;
        }
    }

    let details = if names.is_empty() {
        "no contracts found".to_string()
    } else {
        format!("found {} contracts", names.len())
    };

    let entry = AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "contract_listed",
        contract: None,
        target: None,
        results: None,
        executor: executor(),
        details: Some(&details),
        summary: None,
    };

    if names.is_empty() {
        log_and_print(&entry, "📜 No contracts found in contracts/ directory");
    } else {
        log_and_print(&entry, "📜 Available contracts:");
        for n in names {
            println!("  - {}", n);
        }
    }
}

pub async fn validate(file: &str) {
    let path = if file.ends_with(".toml") {
        format!("contracts/{}", file)
    } else {
        format!("contracts/{}.toml", file)
    };

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<SchemaContracts>(&content) {
            Ok(_) => {
                let entry = AuditLogEntry {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "AUDIT",
                    event: "contract_validated",
                    contract: Some(Contract {
                        name: file,
                        version: "N/A",
                    }),
                    target: None,
                    results: None,
                    executor: executor(),
                    details: Some("syntax valid"),
                    summary: None,
                };
                log_and_print(&entry, &format!("✅ {} is a valid contract", file));
            }
            Err(_) => {
                let entry = AuditLogEntry {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "AUDIT",
                    event: "contract_validated",
                    contract: Some(Contract {
                        name: file,
                        version: "N/A",
                    }),
                    target: None,
                    results: None,
                    executor: executor(),
                    details: Some("invalid syntax"),
                    summary: None,
                };
                log_and_print(&entry, &format!("❌ Invalid contract syntax in {}", file));
            }
        },
        Err(_) => {
            eprintln!("❌ Contract file not found: {}", file);
        }
    }
}

pub async fn show(name: &str) {
    let path = format!("contracts/{}.toml", name);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let entry = AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "contract_shown",
                contract: Some(Contract {
                    name,
                    version: "N/A",
                }),
                target: None,
                results: None,
                executor: executor(),
                details: None,
                summary: None,
            };
            log_and_print(&entry, &format!("📄 Contract: {}", name));
            println!("\n{}", content);
        }
        Err(_) => eprintln!("❌ Contract '{}' not found", name),
    }
}
