use serde::{Deserialize, Serialize};

/// Top-level audit log entry.
/// Each line in audit-YYYY-MM-DD.jsonl will be one of these.
#[derive(Serialize)]
pub struct AuditLogEntry<'a> {
    pub timestamp: String,
    pub level: &'a str,
    pub event: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<Contract<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Target<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<RuleResult>>, // <-- no lifetime here
    pub executor: Executor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ProcessSummary>,
}

/// Contract metadata
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleResult {
    pub column: String,
    pub rule: String,
    pub result: String,
    pub details: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Executor {
    pub user: String,
    pub host: String,
}

/// Summary of a full process run
#[derive(Serialize)]
pub struct ProcessSummary {
    pub contracts_run: usize,
    pub contracts_failed: usize,
    pub status: String, // "SUCCESS" | "FAIL"
}
