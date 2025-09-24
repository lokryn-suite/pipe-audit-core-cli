use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::connectors::from_connection_string_with_profile;
use crate::contracts::load_contract_for_file;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;
use crate::profiles::Profiles;
use polars::prelude::*;
use std::io::Cursor;

/// Validate a single file (local or remote) against its TOML contract
pub async fn validate_file(file_path: &str, profiles: &Profiles) -> Result<()> {
    if file_path.starts_with("s3://")
        || file_path.starts_with("azure://")
        || file_path.starts_with("gcs://")
        || file_path.starts_with("sftp://")
    {
        validate_remote_file(file_path, profiles).await
    } else {
        validate_local_file(file_path, profiles).await
    }
}

/// Validate data string against contracts (for pre-fetched data)
pub async fn validate_data(data: &str, contracts: &crate::contracts::SchemaContracts) -> Result<()> {
    let df = CsvReader::new(Cursor::new(data))
        .finish()
        .context("Failed to parse CSV data")?;
    validate_dataframe(&df, contracts).context("Validation failed")?;
    Ok(())
}

async fn validate_local_file(file_path: &str, _profiles: &Profiles) -> Result<()> {
    let path = PathBuf::from(file_path);
    let contracts = load_contract_for_file(&path);

    let driver = get_driver(&path);
    let df = driver.load(&path).context("Failed to load local file")?;

    println!("🔎 Validating {}", file_path);

    validate_dataframe(&df, &contracts).context("Validation failed")?;
    Ok(())
}

async fn validate_remote_file(file_url: &str, profiles: &Profiles) -> Result<()> {
    println!("🌐 Downloading from remote source: {}", file_url);

    let url = url::Url::parse(file_url)?;
    let path_segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| anyhow::anyhow!("Invalid URL path"))?
        .collect();

    let file_name = path_segments
        .last()
        .ok_or_else(|| anyhow::anyhow!("No filename in URL"))?;

    // Determine file extension for driver selection
    let temp_path = PathBuf::from(file_name);

    let contracts = load_contract_for_file(&temp_path);

    let driver = get_driver(&temp_path);

    let connector = from_connection_string_with_profile(file_url, &contracts.source, profiles).await?;
    let mut reader = connector.fetch(url.path()).await?;

    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("dq_temp_{}", file_name));

    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut buffer)?;
    std::fs::write(&temp_file_path, buffer)?;

    // Load the dataframe from the temporary file
    let df = driver
        .load(&temp_file_path)
        .context("Failed to load downloaded file")?;

    println!("🔎 Validating {}", file_url);

    validate_dataframe(&df, &contracts).context("Validation failed")?;

    if temp_file_path.exists() {
        std::fs::remove_file(&temp_file_path).context("Failed to clean up temporary file")?;
    }

    Ok(())
}
