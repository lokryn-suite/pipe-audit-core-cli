use crate::contracts::SchemaContracts;
use crate::validators::{apply_column_contract, apply_compound_unique, apply_file_contract};
use polars::prelude::*;

/// Validate a DataFrame against its schema contracts
pub fn validate_dataframe(df: &DataFrame, contracts: &SchemaContracts) -> PolarsResult<()> {
    // File-level contracts
    if let Some(file) = &contracts.file {
        for contract in &file.validation {
            apply_file_contract(
                df,
                contract,
                &contracts.contract.name,
                &contracts.contract.version,
            )?;
        }
    }

    // Column-level contracts
    for col in &contracts.columns {
        for contract in &col.validation {
            apply_column_contract(
                df,
                &col.name,
                contract,
                &contracts.contract.name,
                &contracts.contract.name,
            )?;
        }
    }

    // Compound uniqueness contracts
    if let Some(compounds) = &contracts.compound_unique {
        for cu in compounds {
            apply_compound_unique(
                df,
                &cu.columns,
                &contracts.contract.name,
                &contracts.contract.name,
            )?;
        }
    }

    Ok(())
}
