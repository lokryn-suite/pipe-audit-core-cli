use super::Driver;          // Shared trait for all drivers
use anyhow::Result;         // Application-level error handling
use polars::prelude::*;     // Polars DataFrame + ParquetReader/Writer
use std::io::Cursor;        // Wrap &[u8] into a reader

/// Parquet file driver
///
/// Implements the `Driver` trait for Parquet data sources.
/// Loads Parquet data from an in‑memory byte slice into a Polars `DataFrame`.
pub struct ParquetDriver;

impl Driver for ParquetDriver {
    /// Load Parquet data from memory into a DataFrame.
    ///
    /// # Arguments
    /// * `data` - Raw Parquet bytes.
    ///
    /// # Returns
    /// * `Result<DataFrame>` - A Polars DataFrame if parsing succeeds.
    fn load(&self, data: &[u8]) -> Result<DataFrame> {
        let cursor = Cursor::new(data);

        // ParquetReader automatically infers schema and reads into DataFrame
        let df = ParquetReader::new(cursor).finish()?;

        Ok(df)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;

    #[test]
    fn it_loads_parquet_data_from_memory() {
        // Build a small DataFrame in memory
        let mut df = df! (
            "col_a" => &[1, 2, 3],
            "col_b" => &["one", "two", "three"],
        ).unwrap();

        // Serialize to Parquet in a buffer
        let mut buffer: Vec<u8> = Vec::new();
        ParquetWriter::new(&mut buffer)
            .finish(&mut df)
            .expect("Failed to write Parquet to buffer");

        // Load back via driver
        let driver = ParquetDriver;
        let result = driver.load(&buffer);
        assert!(result.is_ok());
        let loaded_df = result.unwrap();
        assert_eq!(loaded_df.shape(), (3, 2));
    }
}
