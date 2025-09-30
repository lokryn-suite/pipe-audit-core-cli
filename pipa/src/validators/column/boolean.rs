// src/validators/column/boolean.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use once_cell::sync::Lazy;
use polars::prelude::*;
use std::collections::HashSet;

// Using Lazy from once_cell to create a static HashSet for efficiency.
static ALLOWED_BOOLEAN_VALUES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["true", "false", "1", "0", "t", "f", "yes", "no", "y", "n"]
        .iter()
        .cloned()
        .collect()
});

pub struct BooleanValidator;

impl Validator for BooleanValidator {
    fn name(&self) -> &'static str {
        "Boolean"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if let Ok(utf8_chunked) = series.str() {
            let bad_count = utf8_chunked
                .into_iter()
                .filter(|opt_val| {
                    if let Some(val) = opt_val {
                        !ALLOWED_BOOLEAN_VALUES.contains(val.to_lowercase().as_str())
                    } else {
                        false // Null values are not considered non-boolean
                    }
                })
                .count();

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
        } else {
            Ok(ValidationReport {
                status: "skipped",
                details: Some("column is not a string type".to_string()),
            })
        }
    }
}
