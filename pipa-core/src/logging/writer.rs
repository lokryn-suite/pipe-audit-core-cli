//! Logging writer.
//!
//! - Writes structured `AuditLogEntry`s to daily JSONL files.
//! - After each write, automatically seals any unsealed logs into the encrypted ledger.
//!   This ensures the ledger is always up to date without a separate cron job.
use chrono::Utc;
use serde_json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::logging::ledger::seal_unsealed_logs;
use crate::logging::schema::AuditLogEntry; // 👈 bring in sealing

/// Ensure `logs/` directory exists.
/// Creates it if missing.
fn ensure_logs_dir() -> PathBuf {
    let dir = PathBuf::from("logs");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("cannot create logs directory");
    }
    dir
}

/// Get today’s log file path in JSONL format.
/// Example: `logs/audit-2025-10-01.jsonl`
fn today_log_path() -> PathBuf {
    let logs_dir = ensure_logs_dir();
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let log_filename = format!("audit-{}.jsonl", today);
    logs_dir.join(log_filename)
}

/// Append an `AuditLogEntry` to today’s JSONL file.
/// Each entry is serialized as one line of JSON.
/// After writing, automatically seal any unsealed logs.
pub fn log_event(entry: &AuditLogEntry) {
    let log_path = today_log_path();

    let json = serde_json::to_string(entry).expect("failed to serialize log entry");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("cannot open daily audit log file");

    writeln!(file, "{}", json).expect("failed to write log entry");

    // 🔒 After writing, seal any unsealed logs (older than today)
    let today = Utc::now().format("%Y-%m-%d").to_string();
    seal_unsealed_logs(&PathBuf::from("logs"), &today);
}

/// Append an `AuditLogEntry` to JSONL *and* print a console message.
///
/// - `entry`: the structured audit log entry (full detail, sealed later)
/// - `console_msg`: a curated, PII‑safe one‑liner for operator visibility
#[allow(dead_code)]
pub fn log_and_print(entry: &AuditLogEntry, console_msg: &str) {
    // Write full detail to audit log + seal unsealed logs
    log_event(entry);

    // Print curated message to console
    println!("{}", console_msg);
}
