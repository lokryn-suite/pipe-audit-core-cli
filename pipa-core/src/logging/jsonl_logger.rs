//! JSONL file logger implementation.
//!
//! This is the default logger used by the CLI. It writes audit logs
//! to daily JSONL files and maintains an encrypted hash ledger for
//! tamper-resistance.

use crate::logging::ledger::seal_unsealed_logs;
use crate::logging::logger_trait::AuditLogger;
use crate::logging::schema::AuditLogEntry;
use chrono::Utc;
use serde_json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// JSONL file-based audit logger.
///
/// Writes audit logs to `logs/audit-YYYY-MM-DD.jsonl` files.
/// After each write, automatically seals any unsealed logs into
/// the encrypted ledger.
pub struct JsonlLogger {
    logs_dir: PathBuf,
}

impl JsonlLogger {
    /// Create a new JSONL logger that writes to the given directory.
    ///
    /// # Arguments
    /// * `logs_dir` - Directory where audit logs will be stored (e.g., "logs")
    ///
    /// # Panics
    /// Panics if the directory cannot be created.
    pub fn new(logs_dir: PathBuf) -> Self {
        // Ensure logs directory exists
        if !logs_dir.exists() {
            fs::create_dir_all(&logs_dir).expect("cannot create logs directory");
        }
        Self { logs_dir }
    }

    /// Create a new JSONL logger with default "logs" directory.
    pub fn default() -> Self {
        Self::new(PathBuf::from("logs"))
    }

    /// Get today's log file path.
    fn today_log_path(&self) -> PathBuf {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let log_filename = format!("audit-{}.jsonl", today);
        self.logs_dir.join(log_filename)
    }
}

impl AuditLogger for JsonlLogger {
    fn log_event(&self, entry: &AuditLogEntry) {
        let log_path = self.today_log_path();

        // Serialize to JSON
        let json = serde_json::to_string(entry).expect("failed to serialize log entry");

        // Append to today's log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("cannot open daily audit log file");

        writeln!(file, "{}", json).expect("failed to write log entry");

        // Seal any unsealed logs (older than today)
        let today = Utc::now().format("%Y-%m-%d").to_string();
        seal_unsealed_logs(&self.logs_dir, &today);
    }

    fn log_and_print(&self, entry: &AuditLogEntry, console_msg: &str) {
        // Write full detail to audit log + seal unsealed logs
        self.log_event(entry);

        // Print curated message to console
        println!("{}", console_msg);
    }
}
