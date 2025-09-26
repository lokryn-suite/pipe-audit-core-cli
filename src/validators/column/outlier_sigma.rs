// src/validators/column/outlier_sigma.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct OutlierSigmaValidator {
    pub sigma: f64,
}

impl Validator for OutlierSigmaValidator {
    fn name(&self) -> &'static str {
        "OutlierSigma"
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

        if let (Some(mean), Some(std_dev)) = (values.mean(), values.std(1)) {
            if std_dev == 0.0 {
                return Ok(ValidationReport {
                    status: "pass",
                    details: Some("standard deviation is zero; no outliers possible".to_string()),
                });
            }

            let deviation = (values - mean).into_series();
            let threshold = self.sigma * std_dev;

            let mask = abs(&deviation)?.gt(threshold);

            // Handle the Result from .gt() before calling .sum()
            let outlier_count = mask?.sum().unwrap_or(0);

            if outlier_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!("outliers={}, sigma={}", outlier_count, self.sigma)),
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
                details: Some("mean or standard deviation could not be calculated".to_string()),
            })
        }
    }
}
