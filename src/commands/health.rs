use crate::logging::schema::{AuditLogEntry, Executor};
use crate::logging::writer::log_and_print;
use crate::profiles::load_profiles;
use chrono::Utc;
use hostname;
use std::path::Path;
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

pub async fn run() {
    let mut healthy = true;

    // Check contracts directory
    if !Path::new("contracts").exists() {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_check",
            contract: None,
            target: None,
            results: None,
            executor: executor(),
            details: Some("contracts directory missing"),
            summary: None,
        };
        log_and_print(&entry, "❌ contracts/ directory not found");
        healthy = false;
    } else {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_check",
            contract: None,
            target: None,
            results: None,
            executor: executor(),
            details: Some("contracts directory exists"),
            summary: None,
        };
        log_and_print(&entry, "✅ contracts/ directory exists");
    }

    // Check logs directory
    if !Path::new("logs").exists() {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_check",
            contract: None,
            target: None,
            results: None,
            executor: executor(),
            details: Some("logs directory missing"),
            summary: None,
        };
        log_and_print(&entry, "❌ logs/ directory not found");
        healthy = false;
    } else {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_check",
            contract: None,
            target: None,
            results: None,
            executor: executor(),
            details: Some("logs directory exists"),
            summary: None,
        };
        log_and_print(&entry, "✅ logs/ directory exists");
    }

    // Check profiles
    match load_profiles() {
        Ok(profiles) => {
            if profiles.is_empty() {
                let entry = AuditLogEntry {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "AUDIT",
                    event: "health_check",
                    contract: None,
                    target: None,
                    results: None,
                    executor: executor(),
                    details: Some("no profiles configured"),
                    summary: None,
                };
                log_and_print(&entry, "⚠️ No profiles configured");
            } else {
                let entry = AuditLogEntry {
                    timestamp: Utc::now().to_rfc3339(),
                    level: "AUDIT",
                    event: "health_check",
                    contract: None,
                    target: None,
                    results: None,
                    executor: executor(),
                    details: Some(&format!("{} profiles loaded", profiles.len())),
                    summary: None,
                };
                log_and_print(&entry, &format!("✅ {} profile(s) loaded", profiles.len()));
            }
        }
        Err(_) => {
            let entry = AuditLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                level: "AUDIT",
                event: "health_check",
                contract: None,
                target: None,
                results: None,
                executor: executor(),
                details: Some("failed to load profiles"),
                summary: None,
            };
            log_and_print(&entry, "❌ Failed to load profiles");
            healthy = false;
        }
    }

    // Final summary
    if healthy {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_summary",
            contract: None,
            target: None,
            results: None,
            executor: executor(),
            details: Some("system healthy"),
            summary: None,
        };
        log_and_print(&entry, "🎉 System healthy");
    } else {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_summary",
            contract: None,
            target: None,
            results: None,
            executor: executor(),
            details: Some("system issues detected"),
            summary: None,
        };
        log_and_print(&entry, "💥 System issues detected. Check logs for details.");
    }
}
