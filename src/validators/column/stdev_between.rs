// src/validators/column/stdev_between.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct StdevBetweenValidator {
    pub min: f64,
    pub max: f64,
}

impl Validator for StdevBetweenValidator {
    fn name(&self) -> &'static str {
        "StdevBetween"
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

        // The argument `1` specifies a sample standard deviation (ddof=1).
        if let Some(std_dev) = values.std(1) {
            if std_dev >= self.min && std_dev <= self.max {
                Ok(ValidationReport {
                    status: "pass",
                    details: None,
                })
            } else {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!(
                        "observed_stdev={:.2}, min={}, max={}",
                        std_dev, self.min, self.max
                    )),
                })
            }
        } else {
            Ok(ValidationReport {
                status: "skipped",
                details: Some("standard deviation could not be calculated".to_string()),
            })
        }
    }
}
