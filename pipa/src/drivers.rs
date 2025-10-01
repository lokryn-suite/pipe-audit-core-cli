// src/drivers.rs

pub mod csv;
pub mod parquet;

use anyhow::{Result, anyhow};
use polars::prelude::*;

/// Trait that all drivers implement to load data from an in-memory byte slice.
pub trait Driver {
    fn load(&self, data: &[u8]) -> Result<DataFrame>;
}

/// Factory function to get the correct driver based on a file extension.
pub fn get_driver(extension: &str) -> Result<Box<dyn Driver>> {
    match extension {
        "csv" => Ok(Box::new(csv::CsvDriver)),
        "parquet" => Ok(Box::new(parquet::ParquetDriver)),
        _ => Err(anyhow!("Unsupported file extension: {}", extension)),
    }
}
