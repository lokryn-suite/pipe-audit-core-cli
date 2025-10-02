use serde::{Deserialize, Serialize};

/// Top-level audit log entry.
/// Each line in `audit-YYYY-MM-DD.jsonl` will be one of these.
///
/// This is the canonical structure for all audit events:
/// - timestamped
/// - leveled (INFO, AUDIT, ERROR, etc.)
/// - typed by `event`
/// - optionally tied to a contract, target, results, or summary
#[derive(Serialize)]
pub struct AuditLogEntry<'a> {
    pub timestamp: String, // RFC3339 timestamp
    pub level: &'a str,    // e.g. "AUDIT", "INFO", "ERROR"
    pub event: &'a str,    // semantic event name ("movement_success", "contract_validated")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<Contract<'a>>, // contract metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Target<'a>>, // file/column/rule context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<RuleResult>>, // validation results (owned, no lifetime)
    pub executor: Executor, // who/where ran this
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<&'a str>, // freeform detail string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ProcessSummary>, // summary of a full run
}

/// Contract metadata (embedded in AuditLogEntry)
#[derive(Serialize)]
pub struct Contract<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

/// Target of validation (file, column, rule)
#[derive(Serialize)]
pub struct Target<'a> {
    pub file: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<&'a str>,
}

/// Result of a single rule evaluation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleResult {
    pub column: String,          // which column was validated
    pub rule: String,            // rule name (e.g. "not_null")
    pub result: String,          // "pass" | "fail"
    pub details: Option<String>, // optional failure details
}

/// Executor metadata (who/where ran the validation)
#[derive(Clone, Serialize)]
pub struct Executor {
    pub user: String, // user ID or system account
    pub host: String, // hostname or container ID
}

/// Summary of a full process run
#[derive(Serialize)]
pub struct ProcessSummary {
    pub contracts_run: usize,
    pub contracts_failed: usize,
    pub status: String, // "SUCCESS" | "FAIL"
}
