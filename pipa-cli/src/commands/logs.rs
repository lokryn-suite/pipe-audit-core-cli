use pipa::logs::{verify_logs,FileStatus} ;

pub async fn verify(date: Option<&str>, _all: bool) {
    let (verification, message) = verify_logs(date);
    println!("{}", message);

    // Per‑file entries
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
