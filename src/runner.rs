use crate::contracts::FileContracts;
use crate::validators::{apply_contract, apply_table_contract};
use polars::prelude::*;
use std::path::Path;

/// Load the TOML contract file that matches the CSV filename
pub fn load_contract_for_csv(csv_path: &str) -> FileContracts {
    let stem = Path::new(csv_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let contract_path = format!("contracts/{}.toml", stem);

    let toml_str = std::fs::read_to_string(&contract_path)
        .unwrap_or_else(|_| panic!("Missing contract file: {}", contract_path));

    toml::from_str(&toml_str).expect("Failed to parse contract TOML")
}

/// Validate a single CSV file against its TOML contract
pub fn validate_file(csv_path: &str) -> PolarsResult<()> {
    let contracts = load_contract_for_csv(csv_path);

    // Read CSV into DataFrame
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(csv_path.into()))?
        .finish()?;

    println!("🔎 Validating {}", csv_path);

    // Table-level contracts
    if let Some(table) = &contracts.table {
        for contract in &table.contracts {
            apply_table_contract(&df, contract)?;
        }
    }

    // Column-level contracts
    for col in &contracts.columns {
        for contract in &col.contracts {
            apply_contract(&df, &col.name, contract)?;
        }
    }

    // // Compound uniqueness
    // if let Some(compounds) = &contracts.compound_unique {
    //     for cu in compounds {
    //         apply_compound_unique(&df, &cu.columns)?;
    //     }
    // }

    Ok(())
}
