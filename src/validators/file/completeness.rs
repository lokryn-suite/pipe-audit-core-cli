// src/validators/file/completeness.rs
use crate::validators::{FileValidator, ValidationReport, ValidationResult};
use anyhow::anyhow;
use polars::prelude::*;

pub struct FileCompletenessValidator {
    pub min_ratio: f64,
}

impl FileValidator for FileCompletenessValidator {
    fn name(&self) -> &'static str {
        "FileCompleteness"
    }

    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport> {
        let total_rows = df.height();
        if total_rows == 0 {
            return Ok(ValidationReport {
                status: "pass",
                details: Some("file is empty".to_string()),
            });
        }

        let masks = df
            .get_columns()
            .iter()
            .map(|s| s.is_not_null())
            .collect::<Vec<_>>();

        let complete_rows_mask = masks
            .into_iter()
            .reduce(|acc, mask| acc & mask)
            .ok_or_else(|| anyhow!("Cannot compute completeness for a file with no columns"))?;

        let complete_rows = complete_rows_mask.sum().unwrap_or(0) as f64;
        let ratio = complete_rows / total_rows as f64;

        if ratio >= self.min_ratio {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("ratio={:.2}, min_ratio={}", ratio, self.min_ratio)),
            })
        }
    }
}
