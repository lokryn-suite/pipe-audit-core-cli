// src/validators/column/type_validator.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct TypeValidator {
    pub dtype: String,
}

impl Validator for TypeValidator {
    fn name(&self) -> &'static str {
        "Type"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;
        
        let actual_dtype = format!("{:?}", series.dtype());

        if actual_dtype == self.dtype {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("expected={}, actual={}", self.dtype, actual_dtype)),
            })
        }
    }
}