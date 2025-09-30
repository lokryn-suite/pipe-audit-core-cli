// src/validators/column/unique.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct UniqueValidator;

impl Validator for UniqueValidator {
    fn name(&self) -> &'static str {
        "Unique"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let unique_count = series.n_unique()?;
        let total_count = series.len();

        if unique_count == total_count {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!(
                    "found {} unique values in {} total rows",
                    unique_count, total_count
                )),
            })
        }
    }
}
