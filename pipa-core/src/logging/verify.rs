//! Ledger sealing + verification
//!
//! - Sealing: decrypt ledger, append hashes for unsealed logs, re-encrypt.
//! - Verification: decrypt ledger, recompute hashes, compare with stored.
//!
//! This ensures logs are tamper-evident: once sealed, any modification
//! or deletion will be detected by verification.
use chrono::{NaiveDate, Utc};
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

use crate::logging::ledger::{compute_sha256, read_ledger_plaintext};

/// Result of verifying a single log file
pub struct FileVerification {
    pub filename: String,
    pub status: FileStatus,
    pub stored_hash: Option<String>,   // hash recorded in ledger
    pub computed_hash: Option<String>, // hash recomputed from file
}

/// Possible verification outcomes for a file
pub enum FileStatus {
    Verified,   // file exists and hash matches ledger
    Mismatched, // file exists but hash differs from ledger
    Missing,    // file referenced in ledger but missing on disk
    Malformed,  // ledger entry malformed (not enough fields)
    Unsealed,   // file exists but not present in ledger
}

/// Aggregated summary across all files checked
pub struct VerificationSummary {
    pub verified: usize,
    pub mismatched: usize,
    pub missing: usize,
    pub malformed: usize,
    pub unsealed: usize,
    pub files: Vec<FileVerification>, // per-file results
}

impl VerificationSummary {
    fn new() -> Self {
        VerificationSummary {
            verified: 0,
            mismatched: 0,
            missing: 0,
            malformed: 0,
            unsealed: 0,
            files: Vec::new(),
        }
    }
}

/// Verify all sealed logs in the encrypted ledger
pub fn verify_all() -> VerificationSummary {
    let logs_dir = PathBuf::from("logs");
    let mut summary = VerificationSummary::new();
    
    // 1. Read all sealed files from the ledger into a HashMap for quick lookups.
    // The map will store: filename -> stored_hash
    let mut sealed_files: HashMap<String, String> = HashMap::new();
    let ledger_plaintext = read_ledger_plaintext();
    if !ledger_plaintext.is_empty() {
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
            } else {
                let filename = parts[1].to_string();
                let stored_hash = parts[2].to_string();
                sealed_files.insert(filename, stored_hash);
            }
        }
    }

    // 2. If the logs directory doesn't exist, we can't find any files.
    // Any files in the ledger at this point must be missing.
    if !logs_dir.exists() {
        for (filename, stored_hash) in sealed_files {
            summary.missing += 1;
            summary.files.push(FileVerification {
                filename,
                status: FileStatus::Missing,
                stored_hash: Some(stored_hash),
                computed_hash: None,
            });
        }
        return summary;
    }

    // 3. Iterate through all log files on disk.
    for entry in fs::read_dir(logs_dir).expect("cannot read logs dir") {
        let path = entry.expect("bad dir entry").path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Check if this file was in our ledger map.
            if let Some(stored_hash) = sealed_files.get(&filename) {
                // The file is sealed. Now, verify the hash.
                let computed_hash = compute_sha256(&path);
                if *stored_hash == computed_hash {
                    summary.verified += 1;
                    summary.files.push(FileVerification {
                        filename: filename.clone(),
                        status: FileStatus::Verified,
                        stored_hash: Some(stored_hash.clone()),
                        computed_hash: Some(computed_hash),
                    });
                } else {
                    summary.mismatched += 1;
                    summary.files.push(FileVerification {
                        filename: filename.clone(),
                        status: FileStatus::Mismatched,
                        stored_hash: Some(stored_hash.clone()),
                        computed_hash: Some(computed_hash),
                    });
                }
                // Remove the file from the map since we've processed it.
                sealed_files.remove(&filename);
            } else {
                // The file exists on disk but was not in the ledger. It's unsealed.
                summary.unsealed += 1;
                summary.files.push(FileVerification {
                    filename,
                    status: FileStatus::Unsealed,
                    stored_hash: None,
                    computed_hash: Some(compute_sha256(&path)), // Still useful to compute hash
                });
            }
        }
    }

    // 4. Any files left in our map were in the ledger but not found on disk.
    // These are missing files.
    for (filename, stored_hash) in sealed_files {
        summary.missing += 1;
        summary.files.push(FileVerification {
            filename,
            status: FileStatus::Missing,
            stored_hash: Some(stored_hash),
            computed_hash: None,
        });
    }

    summary
}
/// Verify logs for a specific date (YYYY-MM-DD). Defaults to yesterday if None.
pub fn verify_date(date: Option<&str>) -> VerificationSummary {
    let logs_dir = PathBuf::from("logs");
    let mut summary = VerificationSummary::new();

    if !logs_dir.exists() {
        return summary;
    }

    // 1. Pick target date
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

    // 2. Decrypt ledger
    let ledger_plaintext = read_ledger_plaintext();
    if ledger_plaintext.is_empty() {
        return summary;
    }
    let ledger_str = String::from_utf8_lossy(&ledger_plaintext);

    // 3. File missing entirely
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

    // 4. File exists, check if sealed in ledger
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
        // File exists but not sealed in ledger
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
