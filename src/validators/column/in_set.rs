// src/validators/column/in_set.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;
use std::collections::HashSet;

pub struct InSetValidator {
    pub values: HashSet<String>,
}

impl Validator for InSetValidator {
    fn name(&self) -> &'static str {
        "InSet"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;
        
        if !series.dtype().is_string() {
            return Ok(ValidationReport {
                status: "skipped",
                details: Some("column is not a string type".to_string()),
            });
        }

        let allowed_values: Vec<String> = self.values.iter().cloned().collect();
        
        // Use the working pattern from our test with lazy DataFrame operations
        let result = df
            .clone()
            .lazy()
            .select([
                col(column_name).is_in(lit(Series::new("allowed".into(), allowed_values)).implode(), false).not()
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