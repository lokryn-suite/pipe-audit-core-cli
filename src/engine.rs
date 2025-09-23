use polars::prelude::*;
use crate::contracts::SchemaContracts;
use crate::validators::{
    apply_column_contract,
    apply_file_contract,
    apply_compound_unique,
};

/// Validate a DataFrame against its schema contracts
pub fn validate_dataframe(
    df: &DataFrame,
    contracts: &SchemaContracts,
) -> PolarsResult<()> {
    // File-level contracts
    if let Some(file) = &contracts.file {
        for contract in &file.contracts {
            apply_file_contract(df, contract, &contracts.name, &contracts.version)?;
        }
    }

    // Column-level contracts
    for col in &contracts.columns {
        for contract in &col.contracts {
            apply_column_contract(
                df,
                &col.name,
                contract,
                &contracts.name,
                &contracts.version,
            )?;
        }
    }

    // Compound uniqueness contracts
    if let Some(compounds) = &contracts.compound_unique {
        for cu in compounds {
            apply_compound_unique(
                df,
                &cu.columns,
                &contracts.name,
                &contracts.version,
            )?;
        }
    }

    Ok(())
}
