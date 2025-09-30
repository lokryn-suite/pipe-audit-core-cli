use chrono::{NaiveDate, Utc};
use std::path::PathBuf;

use crate::logging::ledger::{compute_sha256, read_ledger_plaintext};

/// Result of verifying a single log file
pub struct FileVerification {
    pub filename: String,
    pub status: FileStatus,
    pub stored_hash: Option<String>,
    pub computed_hash: Option<String>,
}

pub enum FileStatus {
    Verified,
    Mismatched,
    Missing,
    Malformed,
    Unsealed,
}

/// Aggregated summary across all files checked
pub struct VerificationSummary {
    pub verified: usize,
    pub mismatched: usize,
    pub missing: usize,
    pub malformed: usize,
    pub unsealed: usize,
    pub files: Vec<FileVerification>,
}

/// Verify all sealed logs in the ledger
pub fn verify_all() -> VerificationSummary {
    let logs_dir = PathBuf::from("logs");
    let mut summary = VerificationSummary {
        verified: 0,
        mismatched: 0,
        missing: 0,
        malformed: 0,
        unsealed: 0,
        files: Vec::new(),
    };

    if !logs_dir.exists() {
        return summary;
    }

    let ledger_plaintext = read_ledger_plaintext();
    if ledger_plaintext.is_empty() {
        return summary;
    }
    let ledger_str = String::from_utf8_lossy(&ledger_plaintext);

    for line in ledger_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            summary.malformed += 1;
            summary.files.push(FileVerification {
                filename: line.to_string(),
                status: FileStatus::Malformed,
                stored_hash: None,
                computed_hash: None,
            });
            continue;
        }
        let filename = parts[1].to_string();
        let stored_hash = parts[2].to_string();
        let log_path = logs_dir.join(&filename);

        if !log_path.exists() {
            summary.missing += 1;
            summary.files.push(FileVerification {
                filename,
                status: FileStatus::Missing,
                stored_hash: Some(stored_hash),
                computed_hash: None,
            });
            continue;
        }

        let computed_hash = compute_sha256(&log_path);
        if stored_hash == computed_hash {
            summary.verified += 1;
            summary.files.push(FileVerification {
                filename,
                status: FileStatus::Verified,
                stored_hash: Some(stored_hash),
                computed_hash: Some(computed_hash),
            });
        } else {
            summary.mismatched += 1;
            summary.files.push(FileVerification {
                filename,
                status: FileStatus::Mismatched,
                stored_hash: Some(stored_hash),
                computed_hash: Some(computed_hash),
            });
        }
    }

    summary
}

/// Verify a single date (YYYY-MM-DD). Defaults to yesterday if None.
pub fn verify_date(date: Option<&str>) -> VerificationSummary {
    let logs_dir = PathBuf::from("logs");
    let mut summary = VerificationSummary {
        verified: 0,
        mismatched: 0,
        missing: 0,
        malformed: 0,
        unsealed: 0,
        files: Vec::new(),
    };

    if !logs_dir.exists() {
        return summary;
    }

    let target_date = match date {
        Some(d) => match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(date) => date.format("%Y-%m-%d").to_string(),
            Err(_) => return summary,
        },
        None => {
            let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
            yesterday.format("%Y-%m-%d").to_string()
        }
    };

    let log_filename = format!("audit-{}.jsonl", target_date);
    let log_path = logs_dir.join(&log_filename);

    let ledger_plaintext = read_ledger_plaintext();
    if ledger_plaintext.is_empty() {
        return summary;
    }
    let ledger_str = String::from_utf8_lossy(&ledger_plaintext);

    if !log_path.exists() {
        summary.missing += 1;
        summary.files.push(FileVerification {
            filename: log_filename,
            status: FileStatus::Missing,
            stored_hash: None,
            computed_hash: None,
        });
        return summary;
    }

    if let Some(line) = ledger_str.lines().find(|l| l.contains(&log_filename)) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            summary.malformed += 1;
            summary.files.push(FileVerification {
                filename: log_filename,
                status: FileStatus::Malformed,
                stored_hash: None,
                computed_hash: None,
            });
            return summary;
        }
        let stored_hash = parts[2].to_string();
        let computed_hash = compute_sha256(&log_path);

        if stored_hash == computed_hash {
            summary.verified += 1;
            summary.files.push(FileVerification {
                filename: log_filename,
                status: FileStatus::Verified,
                stored_hash: Some(stored_hash),
                computed_hash: Some(computed_hash),
            });
        } else {
            summary.mismatched += 1;
            summary.files.push(FileVerification {
                filename: log_filename,
                status: FileStatus::Mismatched,
                stored_hash: Some(stored_hash),
                computed_hash: Some(computed_hash),
            });
        }
    } else {
        summary.unsealed += 1;
        summary.files.push(FileVerification {
            filename: log_filename,
            status: FileStatus::Unsealed,
            stored_hash: None,
            computed_hash: None,
        });
    }

    summary
}
