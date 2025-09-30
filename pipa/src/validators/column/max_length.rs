// src/validators/column/max_length.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct MaxLengthValidator {
    pub value: usize,
}

impl Validator for MaxLengthValidator {
    fn name(&self) -> &'static str {
        "MaxLength"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if let Ok(ca) = series.str() {
            // Correct method name is str_len_chars
            let lengths = ca.str_len_chars();
            let mask = lengths.gt(self.value as u32);
            let bad_count = mask.sum().unwrap_or(0);

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!(
                        "bad_count={}, max_length={}",
                        bad_count, self.value
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
                details: Some("column is not a string type".to_string()),
            })
        }
    }
}
