use pipa::audit_logging::JsonlLogger;
use pipa::logs::{verify_logs, FileStatus};

/// Verify log integrity for a given date or for all logs.
///
/// Delegates to `pipa::logs::verify_logs(date)`, which performs
/// the actual verification of log files. Prints the engine's
/// summary message, then iterates over each file and displays
/// its verification status with a symbol and label.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa logs verify --date YYYY-MM-DD
/// pipa logs verify --all
/// ```
pub async fn verify(date: Option<&str>, _all: bool) {
    // Run verification via engine API
    let logger = JsonlLogger::default();
    let (verification, message) = verify_logs(&logger, date);

    // Print engine-provided summary message
    println!("{}", message);

    // Print per-file verification results
    for file in &verification.files {
        let (symbol, status_str) = match file.status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        println!("{} {} {}", symbol, file.filename, status_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_status_verified_symbol() {
        let status = FileStatus::Verified;
        let (symbol, status_str) = match status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        assert_eq!(symbol, "✅");
        assert_eq!(status_str, "verified");
    }

    #[test]
    fn test_file_status_mismatched_symbol() {
        let status = FileStatus::Mismatched;
        let (symbol, status_str) = match status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        assert_eq!(symbol, "❌");
        assert_eq!(status_str, "mismatched");
    }

    #[test]
    fn test_file_status_missing_symbol() {
        let status = FileStatus::Missing;
        let (symbol, status_str) = match status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        assert_eq!(symbol, "❓");
        assert_eq!(status_str, "missing");
    }

    #[test]
    fn test_file_status_malformed_symbol() {
        let status = FileStatus::Malformed;
        let (symbol, status_str) = match status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        assert_eq!(symbol, "⚠️");
        assert_eq!(status_str, "malformed");
    }

    #[test]
    fn test_file_status_unsealed_symbol() {
        let status = FileStatus::Unsealed;
        let (symbol, status_str) = match status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        assert_eq!(symbol, "🕒");
        assert_eq!(status_str, "unsealed");
    }
}
