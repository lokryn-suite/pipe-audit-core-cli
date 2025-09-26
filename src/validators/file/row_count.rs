// src/validators/file/row_count.rs
use crate::validators::{FileValidator, ValidationReport, ValidationResult};
use polars::prelude::*;

pub struct RowCountValidator {
    pub min: usize,
    pub max: Option<usize>,
}

impl FileValidator for RowCountValidator {
    fn name(&self) -> &'static str {
        "RowCount"
    }

    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport> {
        let rows = df.height();
        let max_check = self.max.map(|m| rows > m).unwrap_or(false);

        if rows < self.min || max_check {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!(
                    "rows={}, min={}, max={:?}",
                    rows, self.min, self.max
                )),
            })
        } else {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        }
    }
}
