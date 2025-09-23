use polars::prelude::*;
use std::fs::File;
use std::path::Path;

use super::DataSource;

pub struct ParquetDriver;

impl DataSource for ParquetDriver {
    fn load(&self, path: &Path) -> PolarsResult<DataFrame> {
        ParquetReader::new(File::open(path)?).finish()
    }
}
