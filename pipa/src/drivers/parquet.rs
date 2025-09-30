// src/drivers/parquet.rs

use super::Driver;
use anyhow::Result;
use polars::prelude::*;
// The `Cursor` is used implicitly by the ParquetReader, but we need to import it here.
use std::io::Cursor;

pub struct ParquetDriver;

impl Driver for ParquetDriver {
    fn load(&self, data: &[u8]) -> Result<DataFrame> {
        let cursor = Cursor::new(data);
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
        let mut df = df! (
            "col_a" => &[1, 2, 3],
            "col_b" => &["one", "two", "three"],
        )
        .unwrap();

        let mut buffer: Vec<u8> = Vec::new();
        ParquetWriter::new(&mut buffer)
            .finish(&mut df)
            .expect("Failed to write Parquet to buffer");

        let driver = ParquetDriver;
        let result = driver.load(&buffer);
        assert!(result.is_ok());
        let loaded_df = result.unwrap();
        assert_eq!(loaded_df.shape(), (3, 2));
    }
}
