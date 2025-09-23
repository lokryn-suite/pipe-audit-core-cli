use std::path::PathBuf;
use polars::prelude::*;

use crate::contracts::load_contract_for_file;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;

/// Validate a single file (CSV or Parquet) against its TOML contract
pub fn validate_file(file_path: &str) -> PolarsResult<()> {
    let path = PathBuf::from(file_path);
    let contracts = load_contract_for_file(&path);

    let driver = get_driver(&path);
    let df = driver.load(&path)?;

    println!("🔎 Validating {}", file_path);

    validate_dataframe(&df, &contracts)
}
