use chrono::Utc;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufReader, Read};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_subscriber::fmt::time::UtcTime;
use whoami;
use hostname;

/// Ensure logs/ exists
fn ensure_logs_dir() -> PathBuf {
    let dir = PathBuf::from("logs");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("cannot create logs directory");
    }
    dir
}

/// Compute SHA256 of a file
fn compute_sha256(path: &PathBuf) -> String {
    let file = File::open(path).expect("cannot open log file for hashing");
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer).expect("failed to read file");
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    format!("{:x}", hasher.finalize())
}

/// Append hash record to hash_ledger.txt
fn append_to_ledger(filename: &str, hash: &str) {
    let ledger_path = ensure_logs_dir().join("hash_ledger.txt");
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)
        .expect("cannot open hash_ledger.txt");
    let line = format!("{} {} {}\n", Utc::now().to_rfc3339(), filename, hash);
    f.write_all(line.as_bytes()).expect("cannot write to ledger");
}

/// Seal all unsealed log files (older than today, not yet in ledger)
fn seal_unsealed_logs(logs_dir: &PathBuf, today: &str) {
    let ledger_path = logs_dir.join("hash_ledger.txt");
    let ledger_contents = fs::read_to_string(&ledger_path).unwrap_or_default();

    for entry in fs::read_dir(logs_dir).expect("cannot read logs dir") {
        let entry = entry.expect("bad dir entry");
        let path = entry.path();
        if path.is_file() {
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname.starts_with("audit-") && fname.ends_with(".jsonl") {
                    // skip today's file
                    if fname.contains(today) { continue; }
                    // skip if already in ledger
                    if ledger_contents.contains(fname) { continue; }

                    // compute hash and append
                    let hash = compute_sha256(&path);
                    append_to_ledger(fname, &hash);
                }
            }
        }
    }
}

/// Initialize logging with daily rotation + sealing
pub fn init_logging() {
    let logs_dir = ensure_logs_dir();
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let log_filename = format!("audit-{}.jsonl", today);
    let log_path = logs_dir.join(&log_filename);

    // Seal any unsealed logs from previous days
    seal_unsealed_logs(&logs_dir, &today);

    // Open today's log file in append mode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("cannot open daily audit log file");

    // Executor identity
    let user = env::var("PIPEAUDIT_EXECUTOR_ID").unwrap_or_else(|_| whoami::username());
    let host = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    // Env filter (default INFO)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // JSON layer writing to daily file
    let file_layer = fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .json()
        .with_writer(file)
        .with_current_span(false)
        .with_span_list(false)
        .with_target(false)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .init();

    info!(user=%user, host=%host, event="startup", "logging initialized");
}
