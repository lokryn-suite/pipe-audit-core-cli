use polars::prelude::*;
use std::path::Path;

use super::DataSource;

pub struct CsvDriver;

impl DataSource for CsvDriver {
    fn load(&self, path: &Path) -> PolarsResult<DataFrame> {
        CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(path.to_path_buf()))?
            .finish()
    }
}
