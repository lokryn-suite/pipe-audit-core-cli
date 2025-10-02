//! Log verification functions for the engine

use crate::engine::log_action;
use crate::logging::verify::{FileVerification, verify_all, verify_date};

/// Result of log verification
pub struct LogVerification {
    pub valid: bool,
    pub verified: usize,
    pub mismatched: usize,
    pub missing: usize,
    pub malformed: usize,
    pub unsealed: usize,
    pub files: Vec<FileVerification>,
}

/// Verify logs for a specific date or all logs
pub fn verify_logs(date: Option<&str>) -> (LogVerification, String) {
    let summary = if let Some(date) = date {
        verify_date(Some(date))
    } else {
        verify_all()
    };

    let all_valid = summary.mismatched == 0
        && summary.missing == 0
        && summary.malformed == 0
        && summary.unsealed == 0;

    let details = format!(
        "valid={}, verified={}, mismatched={}, missing={}, malformed={}, unsealed={}",
        all_valid,
        summary.verified,
        summary.mismatched,
        summary.missing,
        summary.malformed,
        summary.unsealed
    );

    let message = log_action("logs_verified", Some(&details), None, None, None);

    (
        LogVerification {
            valid: all_valid,
            verified: summary.verified,
            mismatched: summary.mismatched,
            missing: summary.missing,
            malformed: summary.malformed,
            unsealed: summary.unsealed,
            files: summary.files,
        },
        message,
    )
}
