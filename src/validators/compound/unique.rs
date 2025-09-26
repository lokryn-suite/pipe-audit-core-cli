// src/validators/compound/unique.rs
use crate::validators::{CompoundValidator, ValidationReport, ValidationResult};
use polars::frame::UniqueKeepStrategy;
use polars::prelude::*;

pub struct CompoundUniqueValidator {
    pub columns: Vec<String>,
}

impl CompoundValidator for CompoundUniqueValidator {
    fn name(&self) -> &'static str {
        "CompoundUnique"
    }

    fn validate(&self, df: &DataFrame) -> ValidationResult<ValidationReport> {
        let total_rows = df.height();

        let unique_df = df.unique_stable(Some(&self.columns), UniqueKeepStrategy::First, None)?;
        let distinct_rows = unique_df.height();

        if distinct_rows == total_rows {
            Ok(ValidationReport {
                status: "pass",
                details: Some(format!("columns={:?}, rows={}", self.columns, total_rows)),
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!(
                    "columns={:?}, rows={}, distinct={}",
                    self.columns, total_rows, distinct_rows
                )),
            })
        }
    }
}
