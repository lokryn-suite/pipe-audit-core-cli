use crate::logging::init::init_logging;
use crate::logging::ledger::ensure_ledger_key_exists;
use std::fs;
use std::path::Path;

pub fn init_project() -> Result<String, Box<dyn std::error::Error>> {

    init_logging();
    ensure_ledger_key_exists();

    let mut actions = Vec::new();

    let contracts_dir = Path::new("contracts");
    fs::create_dir_all(contracts_dir)?;

    if !Path::new(".env").exists() {
        fs::write(".env", "# .env file for project-specific environment variables")?;
        actions.push("Created .env file");
    }

    if !Path::new("profiles.toml").exists() {
        let profile_content = r#"# Pipe Audit profiles.toml example
# provider can be: "local", "s3", "azure", "gcs"
name = "example-profile"
provider = "local"
# For s3/azure/gcs, add provider-specific fields (endpoint, region, connection_string, etc.)
"#;
        fs::write("profiles.toml", profile_content)?;
        actions.push("Created profiles.toml");
    }

    // Use `join` to build paths robustly for any OS.
    let contract_path = contracts_dir.join("example.toml");
    if !contract_path.exists() {
        let contract_content = r#"# contracts/example.toml
[contract]
name = "example-contract"
version = "0.1.0"

[source]
type = "local"
location = "data/example.csv"

[file.validation]
# row_count.min = 1
# completeness.min_ratio = 0.95

[[columns]]
name = "id"
[[columns.validation]]
type = { dtype = "i64" }
[[columns.validation]]
unique = true
"#;
        fs::write(&contract_path, contract_content)?;
        actions.push("Created contracts/example.toml");
    }

    if actions.is_empty() {
        Ok("Project already initialized. No changes were made.".to_string())
    } else {
        Ok(format!("Successfully initialized project. {}", actions.join(". ")))
    }
}