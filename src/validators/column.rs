use polars::prelude::*;
use regex::Regex;
use std::collections::HashSet;

use crate::contracts::ContractType;
use crate::logging::log_validation_event;

pub fn apply_column_contract(
    df: &DataFrame,
    column: &str,
    contract: &ContractType,
    contract_name: &str,
    contract_version: &str,
) -> PolarsResult<()> {
    match contract {
        ContractType::NotNull => {
            let series = df.column(column)?;
            let null_count = series.null_count();
            if null_count > 0 {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "NotNull",
                    "fail",
                    Some(&format!("null_count={}", null_count)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "NotNull",
                    "pass",
                    None,
                );
            }
        }

        ContractType::Unique => {
            let series = df.column(column)?;
            let unique_count = series.n_unique()?;
            if unique_count != series.len() {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Unique",
                    "fail",
                    Some(&format!("unique_count={}", unique_count)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Unique",
                    "pass",
                    None,
                );
            }
        }

        ContractType::Pattern { pattern } => {
            let re =
                Regex::new(pattern).map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;
            let series = df.column(column)?;
            let mut bad_count = 0;
            for opt_val in series.str()?.into_iter() {
                if let Some(val) = opt_val {
                    if !re.is_match(val) {
                        bad_count += 1;
                    }
                }
            }
            if bad_count > 0 {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Pattern",
                    "fail",
                    Some(&format!("bad_count={}, pattern={}", bad_count, pattern)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Pattern",
                    "pass",
                    None,
                );
            }
        }

        ContractType::MaxLength { value } => {
            let series = df.column(column)?;
            let mut bad_count = 0;
            for opt_val in series.str()?.into_iter() {
                if let Some(val) = opt_val {
                    if val.len() > *value {
                        bad_count += 1;
                    }
                }
            }
            if bad_count > 0 {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "MaxLength",
                    "fail",
                    Some(&format!("bad_count={}, max_length={}", bad_count, value)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "MaxLength",
                    "pass",
                    None,
                );
            }
        }

        ContractType::Range { min, max } => {
            let series = df.column(column)?;
            if let Ok(values) = series.i64() {
                let mask = values.lt(*min) | values.gt(*max);
                let bad_count = mask.sum().unwrap_or(0);
                if bad_count > 0 {
                    log_validation_event(
                        contract_name,
                        contract_version,
                        column,
                        "Range",
                        "fail",
                        Some(&format!(
                            "bad_count={}, min={}, max={}",
                            bad_count, min, max
                        )),
                    );
                } else {
                    log_validation_event(
                        contract_name,
                        contract_version,
                        column,
                        "Range",
                        "pass",
                        None,
                    );
                }
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Range",
                    "skipped",
                    Some("not numeric"),
                );
            }
        }

        ContractType::InSet { values } => {
            let allowed: HashSet<_> = values.iter().collect();
            let series = df.column(column)?;
            let mut bad_count = 0;
            for opt_val in series.str()?.into_iter() {
                if let Some(val) = opt_val {
                    if !allowed.contains(&val.to_string()) {
                        bad_count += 1;
                    }
                }
            }
            if bad_count > 0 {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "InSet",
                    "fail",
                    Some(&format!("bad_count={}", bad_count)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "InSet",
                    "pass",
                    None,
                );
            }
        }

        ContractType::Boolean => {
            let allowed = ["true", "false", "1", "0", "yes", "no"];
            let allowed_set: HashSet<_> = allowed.iter().cloned().collect();
            let series = df.column(column)?;
            let mut bad_count = 0;
            for opt_val in series.str()?.into_iter() {
                if let Some(val) = opt_val {
                    if !allowed_set.contains(val.to_lowercase().as_str()) {
                        bad_count += 1;
                    }
                }
            }
            if bad_count > 0 {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Boolean",
                    "fail",
                    Some(&format!("bad_count={}", bad_count)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Boolean",
                    "pass",
                    None,
                );
            }
        }

        ContractType::OutlierSigma { sigma } => {
            let series = df.column(column)?;

            if series.dtype().is_numeric() {
                // First, keep the casted series alive
                let casted = series.cast(&DataType::Float64)?;
                let values = casted.f64().expect("cast to f64 failed");

                let mean = values.mean().unwrap_or(0.0);
                let std = values.std(1).unwrap_or(0.0);

                let abs_vals = values.apply(|opt_v| opt_v.map(|v| (v - mean).abs()));
                let mask = abs_vals.gt(sigma * std);
                let outliers = mask.sum().unwrap_or(0);

                if outliers > 0 {
                    log_validation_event(
                        contract_name,
                        contract_version,
                        column,
                        "OutlierSigma",
                        "fail",
                        Some(&format!("outliers={}, sigma={}", outliers, sigma)),
                    );
                } else {
                    log_validation_event(
                        contract_name,
                        contract_version,
                        column,
                        "OutlierSigma",
                        "pass",
                        None,
                    );
                }
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "OutlierSigma",
                    "skipped",
                    Some("not numeric"),
                );
            }
        }

        ContractType::Distinctness { min_ratio } => {
            let series = df.column(column)?;
            let unique_count = series.n_unique()? as f64;
            let total = series.len() as f64;
            let ratio = if total > 0.0 {
                unique_count / total
            } else {
                0.0
            };

            if ratio < *min_ratio {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Distinctness",
                    "fail",
                    Some(&format!("ratio={:.2}, min_ratio={}", ratio, min_ratio)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Distinctness",
                    "pass",
                    None,
                );
            }
        }

        ContractType::Completeness { min_ratio } => {
            let series = df.column(column)?;
            let non_null = (series.len() - series.null_count()) as f64;
            let total = series.len() as f64;
            let ratio = if total > 0.0 { non_null / total } else { 0.0 };

            if ratio < *min_ratio {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Completeness",
                    "fail",
                    Some(&format!("ratio={:.2}, min_ratio={}", ratio, min_ratio)),
                );
            } else {
                log_validation_event(
                    contract_name,
                    contract_version,
                    column,
                    "Completeness",
                    "pass",
                    None,
                );
            }
        }

        _ => {}
    }
    Ok(())
}
