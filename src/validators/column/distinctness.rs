// src/validators/column/distinctness.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct DistinctnessValidator {
    pub min_ratio: f64,
}

impl Validator for DistinctnessValidator {
    fn name(&self) -> &'static str {
        "Distinctness"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let total_count = series.len();
        if total_count == 0 {
            return Ok(ValidationReport {
                status: "pass",
                details: Some("column is empty".to_string()),
            });
        }

        let unique_count = series.n_unique()? as f64;
        let ratio = unique_count / total_count as f64;

        if ratio >= self.min_ratio {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("ratio={:.2}, min_ratio={}", ratio, self.min_ratio)),
            })
        }
    }
}
