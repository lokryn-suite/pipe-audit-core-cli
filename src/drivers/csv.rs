// src/drivers/csv.rs

use super::Driver;
use anyhow::Result;
use polars::prelude::*;
use polars_io::prelude::CsvReadOptions; // Use the direct import
use polars_io::SerReader;
use std::io::Cursor;

pub struct CsvDriver;

impl Driver for CsvDriver {
    fn load(&self, data: &[u8]) -> Result<DataFrame> {
        let cursor = Cursor::new(data);

        // 1. Create a mutable options struct
        let mut options = CsvReadOptions::default();
        options.has_header = true;

        // 2. Create a reader, THEN apply the options to it
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
