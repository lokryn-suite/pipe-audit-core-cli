// src/validators/column/not_in_set.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;
use std::collections::HashSet;

pub struct NotInSetValidator {
    pub values: HashSet<String>,
}

impl Validator for NotInSetValidator {
    fn name(&self) -> &'static str {
        "NotInSet"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;
        
        if !series.dtype().is_string() {
            return Ok(ValidationReport {
                status: "skipped",
                details: Some("column is not a string type".to_string()),
            });
        }

        let disallowed_values: Vec<String> = self.values.iter().cloned().collect();
        
        // Use the working lazy DataFrame pattern
        // For NotInSet, we want to find values that ARE in the disallowed set (no .not())
        let result = df
            .clone()
            .lazy()
            .select([
                col(column_name).is_in(lit(Series::new("disallowed".into(), disallowed_values)).implode(), false)
            ])
            .collect()?;
        
        let bad_series = result.column(column_name)?;
        let bad_count: u32 = bad_series.bool()?.sum().unwrap_or(0);

        if bad_count > 0 {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("bad_count={}", bad_count)),
            })
        } else {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        }
    }
}