// src/validators/column/mean_between.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct MeanBetweenValidator {
    pub min: f64,
    pub max: f64,
}

impl Validator for MeanBetweenValidator {
    fn name(&self) -> &'static str {
        "MeanBetween"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let f64_series = match series.cast(&DataType::Float64) {
            Ok(s) => s,
            Err(_) => {
                return Ok(ValidationReport {
                    status: "skipped",
                    details: Some("column could not be cast to a numeric type".to_string()),
                });
            }
        };

        let values = f64_series.f64()?;

        if let Some(mean) = values.mean() {
            if mean >= self.min && mean <= self.max {
                Ok(ValidationReport {
                    status: "pass",
                    details: None,
                })
            } else {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!(
                        "observed_mean={:.2}, min={}, max={}",
                        mean, self.min, self.max
                    )),
                })
            }
        } else {
            Ok(ValidationReport {
                status: "skipped",
                details: Some("column contains no non-null values".to_string()),
            })
        }
    }
}
