//! Centralized logging functionality for engine operations

use crate::logging::schema::{AuditLogEntry, Executor};
use crate::logging::writer::log_event;
use chrono::Utc;
use hostname;
use whoami;

/// Centralized logging function for all engine operations
/// Logs full details to crypto-signed audit log and returns PII-safe console message
pub fn log_action(
    event: &str,
    details: Option<&str>,
    contract: Option<&str>,
    version: Option<&str>,
    target: Option<&str>,
) -> String {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    let entry = AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event,
        contract: contract.map(|name| crate::logging::schema::Contract {
            name,
            version: version.unwrap_or("N/A"),
        }),
        target: target.map(|file| crate::logging::schema::Target {
            file,
            column: None,
            rule: None,
        }),
        results: None,
        executor,
        details,
        summary: None,
    };

    log_event(&entry);

    // Return PII-safe message based on event type
    match event {
        "contracts_listed" => "📋 Contracts listed".to_string(),
        "contract_retrieved" => format!("📄 Contract '{}' retrieved", contract.unwrap_or("unknown")),
        "contract_validated" => format!("✅ Contract '{}' validated", contract.unwrap_or("unknown")),
        "profiles_listed" => "👤 Profiles listed".to_string(),
        "profile_tested" => {
            if let Some(detail) = details {
                if detail.contains("connected=true") {
                    format!("✅ Profile '{}' connectivity verified", target.unwrap_or("unknown"))
                } else if detail.contains("exists=true") {
                    format!("❌ Profile '{}' test failed", target.unwrap_or("unknown"))
                } else {
                    format!("❌ Profile '{}' not found", target.unwrap_or("unknown"))
                }
            } else {
                "👤 Profile tested".to_string()
            }
        }
        "logs_verified" => {
            if let Some(detail) = details {
                if detail.contains("valid=true") {
                    "✅ Log integrity verified".to_string()
                } else {
                    "❌ Log integrity check failed".to_string()
                }
            } else {
                "📊 Logs verified".to_string()
            }
        }
        "contract_validation_started" => format!("🚀 Starting validation for '{}'", contract.unwrap_or("unknown")),
        "contract_validation_completed" => format!("✅ Validation completed for '{}'", contract.unwrap_or("unknown")),
        "health_check" => "🏥 Health check completed".to_string(),
        _ => format!("📝 Action: {}", event),
    }
}