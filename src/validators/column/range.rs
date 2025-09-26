// src/validators/column/range.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct RangeValidator {
    pub min: i64,
    pub max: i64,
}

impl Validator for RangeValidator {
    fn name(&self) -> &'static str {
        "Range"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;
        
        if let Ok(values) = series.i64() {
            // Create a boolean mask for values outside the desired range.
            let mask = values.lt(self.min) | values.gt(self.max);
            let bad_count = mask.sum().unwrap_or(0);

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!(
                        "bad_count={}, min={}, max={}",
                        bad_count, self.min, self.max
                    )),
                })
            } else {
                Ok(ValidationReport {
                    status: "pass",
                    details: None,
                })
            }
        } else {
            Ok(ValidationReport {
                status: "skipped",
                details: Some("column is not an integer type".to_string()),
            })
        }
    }
}