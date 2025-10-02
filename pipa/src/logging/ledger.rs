use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use chrono::Utc;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

const LEDGER_PATH: &str = "logs/hash_ledger.enc";

/// Location of the AES key (~/.lokryn/pipeaudit/ledger.key)
fn ledger_key_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".lokryn/pipeaudit/ledger.key")
}

/// Ensure the ledger key exists, creating it securely if missing.
/// - 32 random bytes (AES‑256 key)
/// - Stored with 0600 permissions on Unix
pub fn ensure_ledger_key_exists() {
    let key_path = ledger_key_path();
    if !key_path.exists() {
        fs::create_dir_all(key_path.parent().unwrap()).expect("cannot create key directory");
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        fs::write(&key_path, &key).expect("cannot write ledger key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path).unwrap().permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms).unwrap();
        }
    }
}

/// Load the AES key from disk
fn load_ledger_key() -> Key<Aes256Gcm> {
    let key_bytes = fs::read(ledger_key_path()).expect("missing ledger key");
    assert!(key_bytes.len() == 32, "ledger key must be 32 bytes for AES-256-GCM");
    Key::<Aes256Gcm>::from_slice(&key_bytes).clone()
}

/// Compute SHA‑256 hash of a file (used for sealing logs)
pub fn compute_sha256(path: &PathBuf) -> String {
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

/// Read and decrypt the ledger; returns plaintext bytes (or empty if not present)
pub fn read_ledger_plaintext() -> Vec<u8> {
    if !PathBuf::from(LEDGER_PATH).exists() {
        return Vec::new();
    }
    let data = fs::read(LEDGER_PATH).expect("cannot read encrypted ledger");
    if data.len() < 12 {
        panic!("encrypted ledger corrupted (nonce missing)");
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(&load_ledger_key());
    cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
          .expect("ledger decryption failed")
}

/// Encrypt and write the full plaintext ledger
fn write_ledger_plaintext(plaintext: &[u8]) {
    let cipher = Aes256Gcm::new(&load_ledger_key());
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext)
                           .expect("ledger encryption failed");

    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    fs::write(LEDGER_PATH, out).expect("cannot write encrypted ledger");
}

/// Append a line to the encrypted ledger
/// Format: `<timestamp> <filename> <sha256>\n`
pub fn append_to_ledger(filename: &str, hash: &str) {
    let mut ledger = read_ledger_plaintext();
    let line = format!("{} {} {}\n", Utc::now().to_rfc3339(), filename, hash);
    ledger.extend_from_slice(line.as_bytes());
    write_ledger_plaintext(&ledger);
}

/// Check if a filename is already present in the ledger
pub fn ledger_contains(filename: &str) -> bool {
    let ledger = read_ledger_plaintext();
    let s = String::from_utf8_lossy(&ledger);
    s.contains(filename)
}

/// Seal all unsealed log files (older than today, not yet in encrypted ledger)
pub fn seal_unsealed_logs(logs_dir: &PathBuf, today: &str) {
    for entry in fs::read_dir(logs_dir).expect("cannot read logs dir") {
        let entry = entry.expect("bad dir entry");
        let path = entry.path();
        if path.is_file() {
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname.starts_with("audit-") && fname.ends_with(".jsonl") {
                    if fname.contains(today) { continue; }
                    if ledger_contains(fname) { continue; }
                    let hash = compute_sha256(&path);
                    append_to_ledger(fname, &hash);
                }
            }
        }
    }
}
