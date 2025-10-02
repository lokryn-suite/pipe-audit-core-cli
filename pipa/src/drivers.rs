pub mod csv;
pub mod parquet;

use anyhow::{Result, anyhow};
use polars::prelude::*;

/// Trait that all drivers implement to load data from an in-memory byte slice.
///
/// This abstraction allows the engine to support multiple file formats
/// (CSV, Parquet, etc.) behind a uniform interface. Each driver is responsible
/// for parsing raw bytes into a Polars `DataFrame`.
pub trait Driver {
    fn load(&self, data: &[u8]) -> Result<DataFrame>;
}

/// Factory function to get the correct driver based on a file extension.
///
/// # Arguments
/// * `extension` - File extension string (e.g., `"csv"`, `"parquet"`).
///
/// # Returns
/// * `Box<dyn Driver>` - A boxed driver implementing the `Driver` trait.
///
/// # Errors
/// Returns an error if the extension is unsupported.
pub fn get_driver(extension: &str) -> Result<Box<dyn Driver>> {
    match extension {
        "csv" => Ok(Box::new(csv::CsvDriver)),
        "parquet" => Ok(Box::new(parquet::ParquetDriver)),
        _ => Err(anyhow!("Unsupported file extension: {}", extension)),
    }
}
