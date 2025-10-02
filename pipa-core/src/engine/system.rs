use crate::logging::log_event;
use crate::logging::schema::{AuditLogEntry, Executor};
use crate::logging::writer::log_and_print;
use crate::profiles::load_profiles;
use chrono::Utc;
use hostname;
use std::path::Path as StdPath;
use whoami;

#[derive(Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub contracts_dir_exists: bool,
    pub logs_dir_exists: bool,
    pub profile_count: usize,
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

pub fn run_health_check(log_to_console: bool) -> (HealthStatus, String) {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

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
    let message = crate::engine::log_action("health_check", Some(summary_msg), None, None, None);
    if log_to_console {
        println!("{}", message);
    }

    (status, message)
}
