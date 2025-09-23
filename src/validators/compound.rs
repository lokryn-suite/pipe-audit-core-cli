use crate::logging::log_validation_event;
use polars::prelude::*;
use polars::frame::UniqueKeepStrategy;

/// Apply compound uniqueness across multiple columns
pub fn apply_compound_unique(
    df: &DataFrame,
    cols: &[String],
    contract_name: &str,
    contract_version: &str,
) -> PolarsResult<()> {
    let total = df.height();

    // Get a DataFrame of unique rows across the specified columns
    let unique = df.unique_stable(Some(cols), UniqueKeepStrategy::First, None)?;
    let distinct = unique.height();

    if distinct != total {
        log_validation_event(
            contract_name,
            contract_version,
            "file",
            "CompoundUnique",
            "fail",
            Some(&format!(
                "columns={:?}, rows={}, distinct={}",
                cols, total, distinct
            )),
        );
    } else {
        log_validation_event(
            contract_name,
            contract_version,
            "file",
            "CompoundUnique",
            "pass",
            Some(&format!("columns={:?}, rows={}", cols, total)),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_duplicate_combinations() {
        let df = df![
            "household_id" => &[1, 1, 2],
            "person_id"    => &[1, 1, 2]
        ].unwrap();

        // Should run without panic
        let result = apply_compound_unique(
            &df,
            &vec!["household_id".to_string(), "person_id".to_string()],
            "test_contract",
            "0.1.0"
        );

        assert!(result.is_ok());
        // Later: capture logs or return a status to assert fail/pass
    }
}