use super::ledger::ensure_ledger_key_exists;

pub fn init_logging() {
    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    ensure_ledger_key_exists()
}
