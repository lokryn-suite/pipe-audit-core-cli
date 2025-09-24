use std::path::PathBuf;
use anyhow::{Context, Result};

use crate::contracts::load_contract_for_file;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;
use crate::connectors::from_connection_string;

/// Validate a single file (local or remote) against its TOML contract
pub async fn validate_file(file_path: &str) -> Result<()> {
    // Check if this is a remote URL or local file
    if file_path.starts_with("s3://") || file_path.starts_with("azure://") || 
       file_path.starts_with("gcs://") || file_path.starts_with("sftp://") {
        validate_remote_file(file_path).await
    } else {
        validate_local_file(file_path).await
    }
}

async fn validate_local_file(file_path: &str) -> Result<()> {
    let path = PathBuf::from(file_path);
    let contracts = load_contract_for_file(&path);

    let driver = get_driver(&path);
    let df = driver.load(&path)
        .context("Failed to load local file")?;

    println!("🔎 Validating {}", file_path);

    validate_dataframe(&df, &contracts)
        .context("Validation failed")?;
    Ok(())
}

async fn validate_remote_file(file_url: &str) -> Result<()> {
    println!("🌐 Downloading from remote source: {}", file_url);
    
    // Parse the URL to determine file type
    let url = url::Url::parse(file_url)?;
    let path_segments: Vec<&str> = url.path_segments()
        .ok_or_else(|| anyhow::anyhow!("Invalid URL path"))?
        .collect();
    
    let file_name = path_segments.last()
        .ok_or_else(|| anyhow::anyhow!("No filename in URL"))?;
    
    // Determine file extension for driver selection
    let temp_path = PathBuf::from(file_name);
    
    // Load contracts based on filename
    let contracts = load_contract_for_file(&temp_path);

    // Get the appropriate driver
    let driver = get_driver(&temp_path);

    // Get connector and fetch the file data
    let connector = from_connection_string(file_url).await?;
    let mut reader = connector.fetch(url.path()).await?;

    // Create a temporary file to store the downloaded data
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("dq_temp_{}", file_name));

    // Read all data and write to temp file
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut buffer)?;
    std::fs::write(&temp_file_path, buffer)?;

    // Load the dataframe from the temporary file
    let df = driver.load(&temp_file_path)
        .context("Failed to load downloaded file")?;

    println!("🔎 Validating {}", file_url);

    // Validate the dataframe
    validate_dataframe(&df, &contracts)
        .context("Validation failed")?;

    // Clean up the temporary file
    if temp_file_path.exists() {
        std::fs::remove_file(&temp_file_path)
            .context("Failed to clean up temporary file")?;
    }

    Ok(())
}
