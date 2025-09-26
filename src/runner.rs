// src/runner.rs

use crate::contracts::SchemaContracts;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;
use anyhow::{Context, Result};

/// Validate data bytes against contracts using the appropriate driver
pub async fn validate_data(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
) -> Result<()> {
    println!(
        "🔍 Starting validation with {} bytes, extension: {}",
        data.len(),
        extension
    );

    let driver =
        get_driver(extension).context("Failed to find a suitable driver for the extension")?; //TODO add to logging
    println!("✅ Found driver for extension: {}", extension);

    let df = driver
        .load(data)
        .context("Failed to parse data from memory")?;
    println!(
        "✅ Parsed DataFrame with {} rows, {} columns",
        df.height(),
        df.width()
    );

    validate_dataframe(&df, contracts).context("Validation failed")?;
    println!("✅ Validation completed");

    Ok(())
}
