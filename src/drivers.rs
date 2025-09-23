pub mod csv;
pub mod parquet;

use polars::prelude::*;
use std::path::Path;

/// Trait that all drivers implement
pub trait DataSource {
    fn load(&self, path: &Path) -> PolarsResult<DataFrame>;
}

pub fn get_driver(path: &Path) -> Box<dyn DataSource> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("csv") => Box::new(csv::CsvDriver),
        Some("parquet") => Box::new(parquet::ParquetDriver),
        _ => panic!("Unsupported file type: {:?}", path),
    }
}
