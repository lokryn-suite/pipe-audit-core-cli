// src/validators/column/date_format.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use chrono::NaiveDateTime;
use polars::prelude::*;

pub struct DateFormatValidator {
    pub format: String,
}

impl Validator for DateFormatValidator {
    fn name(&self) -> &'static str {
        "DateFormat"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;
        
        if let Ok(utf8_chunked) = series.str() {
            let bad_count = utf8_chunked
                .into_iter()
                .filter(|opt_val| {
                    if let Some(val) = opt_val {
                        NaiveDateTime::parse_from_str(val, &self.format).is_err()
                    } else {
                        false // Null values don't fail a format match
                    }
                })
                .count();

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!("bad_count={}, format={}", bad_count, self.format)),
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