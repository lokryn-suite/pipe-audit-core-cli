// src/validators/column/pattern.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;
use regex::Regex;

pub struct PatternValidator {
    pub pattern: String,
}

impl Validator for PatternValidator {
    fn name(&self) -> &'static str {
        "Pattern"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let re = Regex::new(&self.pattern)?;

        if let Ok(utf8_chunked) = series.str() {
            let bad_count = utf8_chunked
                .into_iter()
                .filter(|opt_val| {
                    if let Some(val) = opt_val {
                        !re.is_match(val)
                    } else {
                        false // Null values don't fail a pattern match
                    }
                })
                .count();

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!("bad_count={}, pattern={}", bad_count, self.pattern)),
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
