use crate::logging::schema::{AuditLogEntry, Executor, Target};
use crate::logging::verify::{verify_all, verify_date, FileStatus};
use crate::logging::writer::log_and_print;
use chrono::Utc;
use hostname;
use whoami;

fn executor() -> Executor {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Executor {
        user: whoami::username(),
        host: hostname,
    }
}

pub async fn verify(date: Option<&str>, all: bool) {
    let summary = if all { verify_all() } else { verify_date(date) };

    // Overall summary entry
    let entry = AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "ledger_verification_summary",
        contract: None,
        target: None,
        results: None,
        executor: executor(),
        details: Some(&format!(
            "verified={}, mismatched={}, missing={}, malformed={}, unsealed={}",
            summary.verified,
            summary.mismatched,
            summary.missing,
            summary.malformed,
            summary.unsealed
        )),
        summary: None,
    };
    log_and_print(
        &entry,
        &format!(
            "📊 Verification summary:\n   ✅ Verified:   {}\n   ❌ Mismatched: {}\n   ❓ Missing:    {}\n   ⚠️  Malformed:  {}\n   🕒 Unsealed:   {}",
            summary.verified, summary.mismatched, summary.missing, summary.malformed, summary.unsealed
        ),
    );

    // Per‑file entries
    for file in summary.files {
        let (symbol, status_str) = match file.status {
            FileStatus::Verified => ("✅", "verified"),
            FileStatus::Mismatched => ("❌", "mismatched"),
            FileStatus::Missing => ("❓", "missing"),
            FileStatus::Malformed => ("⚠️", "malformed"),
            FileStatus::Unsealed => ("🕒", "unsealed"),
        };

        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "AUDIT",
            event: "ledger_file_status",
            contract: None,
            target: Some(Target {
                file: &file.filename,
                column: None,
                rule: None,
            }),
            results: None,
            executor: executor(),
            details: Some(&format!("status={}", status_str)),
            summary: None,
        };
        log_and_print(
            &entry,
            &format!("{} {} {}", symbol, file.filename, status_str),
        );
    }
}
