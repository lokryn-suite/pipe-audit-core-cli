use chrono::{NaiveDate, Utc};
use std::fs;
use std::path::PathBuf;

pub async fn verify(date: Option<&str>) {
    let logs_dir = PathBuf::from("logs");

    if !logs_dir.exists() {
        eprintln!("❌ logs/ directory not found");
        return;
    }

    let target_date = match date {
        Some(d) => match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(date) => date.format("%Y-%m-%d").to_string(),
            Err(_) => {
                eprintln!("❌ Invalid date format. Use YYYY-MM-DD");
                return;
            }
        },
        None => {
            let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
            yesterday.format("%Y-%m-%d").to_string()
        }
    };

    let log_filename = format!("audit-{}.jsonl", target_date);
    let log_path = logs_dir.join(&log_filename);
    let ledger_path = logs_dir.join("hash_ledger.txt");

    if !log_path.exists() {
        eprintln!("❌ Log file for {} not found", target_date);
        return;
    }

    if !ledger_path.exists() {
        eprintln!("❌ Hash ledger not found");
        return;
    }

    let ledger_contents = match fs::read_to_string(&ledger_path) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("❌ Failed to read hash ledger. Check logs for details.");
            return;
        }
    };

    if ledger_contents.contains(&log_filename) {
        println!("✅ Log file for {} is sealed and verified", target_date);
    } else {
        println!("⚠️  Log file for {} not yet sealed", target_date);
    }
}
