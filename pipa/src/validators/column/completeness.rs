// src/validators/column/completeness.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct CompletenessValidator {
    pub min_ratio: f64,
}

impl Validator for CompletenessValidator {
    fn name(&self) -> &'static str {
        "Completeness"
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

        let non_null_count = (total_count - series.null_count()) as f64;
        let ratio = non_null_count / total_count as f64;

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
