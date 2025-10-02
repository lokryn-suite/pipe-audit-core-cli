use super::Driver; // Trait that all drivers must implement
use anyhow::Result; // Standardized error handling
use polars::prelude::*; // Core Polars DataFrame types
use polars_io::SerReader; // Trait for finishing a reader into a DataFrame
use polars_io::prelude::CsvReadOptions; // Explicit import of CSV options
use std::io::Cursor; // Wraps &[u8] into a reader

/// CSV file driver
///
/// Implements the `Driver` trait for CSV data sources.
/// Loads CSV data from an in‑memory byte slice into a Polars `DataFrame`.
pub struct CsvDriver;

impl Driver for CsvDriver {
    /// Load CSV data from memory into a DataFrame.
    ///
    /// # Arguments
    /// * `data` - Raw CSV bytes (UTF‑8 encoded).
    ///
    /// # Returns
    /// * `Result<DataFrame>` - A Polars DataFrame if parsing succeeds.
    fn load(&self, data: &[u8]) -> Result<DataFrame> {
        let cursor = Cursor::new(data);

        // Configure CSV reader options
        let mut options = CsvReadOptions::default();
        options.has_header = true;

        // Build a reader with options and finish into a DataFrame
        let df = CsvReader::new(cursor).with_options(options).finish()?;

        Ok(df)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_loads_csv_data_from_memory() {
        let csv_data = "col_a,col_b\n1,one\n2,two\n3,three";
        let driver = CsvDriver;
        let result = driver.load(csv_data.as_bytes());
        assert!(result.is_ok());
        let df = result.unwrap();
        assert_eq!(df.shape(), (3, 2));
    }
}
