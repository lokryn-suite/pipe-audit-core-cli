use crate::contracts::ContractType;
use polars::prelude::*;
use regex::Regex;
use std::collections::HashSet;

/// Apply a column-level contract
pub fn apply_contract(df: &DataFrame, column: &str, contract: &ContractType) -> PolarsResult<()> {
    match contract {
        // NotNull
        ContractType::NotNull => {
            let series = df.column(column)?;
            let null_count = series.null_count();
            if null_count > 0 {
                println!("❌ [{}] {} null values", column, null_count);
            } else {
                println!("✅ [{}] NotNull passed", column);
            }
        }

        // Unique
        ContractType::Unique => {
            let series = df.column(column)?;
            let unique_count = series.n_unique()?;
            if unique_count != series.len() {
                println!("❌ [{}] duplicates found", column);
            } else {
                println!("✅ [{}] Unique passed", column);
            }
        }

        // Pattern
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
                println!("❌ [{}] {} values failed Pattern ({})", column, bad_count, pattern);
            } else {
                println!("✅ [{}] Pattern ({}) passed", column, pattern);
            }
        }

        // MaxLength
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
                println!("❌ [{}] {} values exceeded length {}", column, bad_count, value);
            } else {
                println!("✅ [{}] MaxLength (≤ {}) passed", column, value);
            }
        }

        // Range (numeric)
        ContractType::Range { min, max } => {
            let series = df.column(column)?;
            if let Ok(values) = series.i64() {
                let mask = values.lt(*min) | values.gt(*max);
                let bad_count = mask.sum().unwrap_or(0);
                if bad_count > 0 {
                    println!("❌ [{}] {} values outside {}–{}", column, bad_count, min, max);
                } else {
                    println!("✅ [{}] Range {}–{} passed", column, min, max);
                }
            } else {
                println!("⚠️ [{}] not numeric, Range skipped", column);
            }
        }

        // InSet
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
                println!("❌ [{}] {} values not in {:?}", column, bad_count, values);
            } else {
                println!("✅ [{}] InSet {:?} passed", column, values);
            }
        }

        // Boolean
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
                println!("❌ [{}] {} values not boolean-like", column, bad_count);
            } else {
                println!("✅ [{}] Boolean passed", column);
            }
        }

        // OutlierSigma (vectorized)
        ContractType::OutlierSigma { sigma } => {
            let series = df.column(column)?;
            if let Ok(values) = series.f64() {
                let mean = values.mean().unwrap_or(0.0);
                let std = values.std(1).unwrap_or(0.0);

                // abs() is not directly available, so map
                let abs_vals = values.apply(|opt_v| opt_v.map(|v| (v - mean).abs()));
                let mask = abs_vals.gt(sigma * std);
                let outliers = mask.sum().unwrap_or(0);

                if outliers > 0 {
                    println!("❌ [{}] {} outliers (> {}σ from mean {:.2})", column, outliers, sigma, mean);
                } else {
                    println!("✅ [{}] OutlierSigma (±{}σ) passed", column, sigma);
                }
            } else {
                println!("⚠️ [{}] not numeric, OutlierSigma skipped", column);
            }
        }

        // Distinctness (% unique)
        ContractType::Distinctness { min_ratio } => {
            let series = df.column(column)?;
            let unique_count = series.n_unique()? as f64;
            let total = series.len() as f64;
            let ratio = if total > 0.0 { unique_count / total } else { 0.0 };

            if ratio < *min_ratio {
                println!("❌ [{}] Distinctness {:.2}% < min {:.2}%", column, ratio * 100.0, min_ratio * 100.0);
            } else {
                println!("✅ [{}] Distinctness {:.2}% passed", column, ratio * 100.0);
            }
        }

        // Completeness (% non-null)
        ContractType::Completeness { min_ratio } => {
            let series = df.column(column)?;
            let non_null = (series.len() - series.null_count()) as f64;
            let total = series.len() as f64;
            let ratio = if total > 0.0 { non_null / total } else { 0.0 };

            if ratio < *min_ratio {
                println!("❌ [{}] Completeness {:.2}% < min {:.2}%", column, ratio * 100.0, min_ratio * 100.0);
            } else {
                println!("✅ [{}] Completeness {:.2}% passed", column, ratio * 100.0);
            }
        }

        _ => {}
    }
    Ok(())
}

/// Table-level contracts
pub fn apply_table_contract(df: &DataFrame, contract: &ContractType) -> PolarsResult<()> {
    match contract {
        ContractType::RowCount { min, max } => {
            let rows = df.height();
            if rows < *min || max.map(|m| rows > m).unwrap_or(false) {
                println!("❌ [table] RowCount {} outside {}–{:?}", rows, min, max);
            } else {
                println!("✅ [table] RowCount {} within {}–{:?}", rows, min, max);
            }
        }
        _ => {
            println!("⚠️ [table] Contract {:?} not implemented yet", contract);
        }
    }
    Ok(())
}

// pub fn apply_compound_unique(df: &DataFrame, cols: &[String]) -> PolarsResult<()> {
//     // Convert &[String] → Vec<&str>
//     let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();

//     // Deduplicate based on those columns
//     let unique = df.unique(
//         Some(&col_refs),                // &[&str]
//         UniqueKeepStrategy::First,
//         None,
//     )?;

//     if unique.height() != df.height() {
//         println!("❌ [table] Compound uniqueness failed on {:?}", cols);
//     } else {
//         println!("✅ [table] Compound uniqueness passed on {:?}", cols);
//     }
//     Ok(())
// }








