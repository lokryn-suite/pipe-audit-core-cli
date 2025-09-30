use super::orchestration::check_system_health;
use crate::logging::schema::{AuditLogEntry, Executor};
use crate::logging::writer::log_and_print;
use chrono::Utc;
use hostname;
use whoami;

pub async fn run() {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    let status = check_system_health();

    if status.contracts_dir_exists {
        log_and_print(
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
        log_and_print(
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

    log_and_print(
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

    log_and_print(
        &AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "health_summary",
            contract: None,
            target: None,
            results: None,
            executor,
            details: Some(summary_msg),
            summary: None,
        },
        &format!("📊 System status: {}", summary_msg),
    );
}