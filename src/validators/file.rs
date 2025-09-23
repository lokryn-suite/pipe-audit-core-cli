use crate::contracts::ContractType;
use crate::logging::log_validation_event;
use polars::prelude::*;

/// Apply a file-level contract (e.g. row count, completeness)
pub fn apply_file_contract(
    df: &DataFrame,
    contract: &ContractType,
    contract_name: &str,
    contract_version: &str,
) -> PolarsResult<()> {
    match contract {
        ContractType::RowCount { min, max } => {
            let rows = df.height();
            if rows < *min || max.map(|m| rows > m).unwrap_or(false) {
                log_validation_event(
                    contract_name,
                    contract_version,
                    "file",
                    "RowCount",
                    "fail",
                    Some(&format!("rows={}, min={}, max={:?}", rows, min, max)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    "file",
                    "RowCount",
                    "pass",
                    None,
                );
            }
        }

        ContractType::Completeness { min_ratio } => {
            let total = df.height();
            let mut complete_rows = 0;

            for idx in 0..total {
                let mut row_complete = true;
                for series in df.get_columns() {
                    match series.get(idx) {
                        Ok(val) => {
                            if val.is_null() {
                                row_complete = false;
                                break;
                            }
                        }
                        Err(_) => {
                            row_complete = false;
                            break;
                        }
                    }
                }
                if row_complete {
                    complete_rows += 1;
                }
            }

            let ratio = if total > 0 {
                complete_rows as f64 / total as f64
            } else {
                0.0
            };
            if ratio < *min_ratio {
                log_validation_event(
                    contract_name,
                    contract_version,
                    "file",
                    "Completeness",
                    "fail",
                    Some(&format!("ratio={:.2}, min_ratio={}", ratio, min_ratio)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    "file",
                    "Completeness",
                    "pass",
                    None,
                );
            }
        }

        _ => {
            log_validation_event(
                contract_name,
                contract_version,
                "file",
                "Unknown",
                "skipped",
                None,
            );
        }
    }
    Ok(())
}
