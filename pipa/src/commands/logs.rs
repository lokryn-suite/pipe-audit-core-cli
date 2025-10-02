use pipa::logs::{verify_logs, FileStatus};

/// Verify log integrity for a given date or for all logs.
///
/// Delegates to `pipa::logs::verify_logs(date)`, which performs
/// the actual verification of log files. Prints the engine’s
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
    let (verification, message) = verify_logs(date);

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
