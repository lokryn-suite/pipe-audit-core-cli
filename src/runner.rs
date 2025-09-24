use anyhow::{Context, Result};

use crate::contracts::SchemaContracts;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;
use std::io::Write;

/// Validate data bytes against contracts using the appropriate driver
pub async fn validate_data(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
) -> Result<()> {
    let driver = get_driver(std::path::Path::new(&format!("temp.{}", extension)));

    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("dq_temp_{}.{}", std::process::id(), extension));

    let mut file = std::fs::File::create(&temp_file_path)?;
    file.write_all(data)?;
    file.flush()?;

    let df = driver
        .load(&temp_file_path)
        .context("Failed to parse data")?;
    validate_dataframe(&df, contracts).context("Validation failed")?;

    if temp_file_path.exists() {
        std::fs::remove_file(&temp_file_path).context("Failed to clean up temporary file")?;
    }

    Ok(())
}
