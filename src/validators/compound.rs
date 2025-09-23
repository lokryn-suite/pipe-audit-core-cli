use polars::prelude::*;
use super::logging::log_validation_event;

pub fn apply_compound_unique(
    df: &DataFrame,
    cols: &[String],
    contract_name: &str,
    contract_version: &str,
) -> PolarsResult<()> {
    let subset = df.select(cols)?;
    let unique = subset.unique(None, UniqueKeepStrategy::First, None)?;

    if unique.height() != subset.height() {
        log_validation_event(
            contract_name,
            contract_version,
            "file",
            "CompoundUnique",
            "fail",
            Some(&format!("columns={:?}, rows={}, unique={}", cols, subset.height(), unique.height())),
        );
    } else {
        log_validation_event(
            contract_name,
            contract_version,
            "file",
            "CompoundUnique",
            "pass",
            Some(&format!("columns={:?}, rows={}", cols, subset.height())),
        );
    }

    Ok(())
}
