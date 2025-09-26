use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct NotNullValidator;

impl Validator for NotNullValidator {
    fn name(&self) -> &'static str {
        "NotNull"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let null_count = series.null_count();

        if null_count > 0 {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("null_count={}", null_count)),
            })
        } else {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        }
    }
}
