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

        let f64_series = match series.strict_cast(&DataType::Float64) {
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
#[cfg(test)]
mod tests {
    use super::*;

    fn make_f64_df(values: &[Option<f64>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    fn make_str_df(values: &[Option<&str>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    #[test]
    fn passes_when_no_outliers() {
        // mean = 5, std_dev = ~1.41, threshold = 2*1.41 = 2.82
        // values are within 2.82 of mean
        let df = make_f64_df(&[Some(4.0), Some(5.0), Some(6.0)]);
        let validator = OutlierSigmaValidator { sigma: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_when_outliers_present() {
        // Eight normals around 10 and one extreme at 100.
        // mean ≈ 20.0, sample std ≈ 30.0, threshold = 2σ ≈ 60.0
        // deviation of 100 from mean ≈ 80.0 > 60.0 → outlier flagged.
        let df = make_f64_df(&[
            Some(10.0),
            Some(10.0),
            Some(10.0),
            Some(10.0),
            Some(10.0),
            Some(10.0),
            Some(10.0),
            Some(10.0),
            Some(100.0),
        ]);
        let validator = OutlierSigmaValidator { sigma: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("outliers="));
    }

    #[test]
    fn passes_when_std_dev_zero() {
        // all values identical → std_dev = 0
        let df = make_f64_df(&[Some(5.0), Some(5.0), Some(5.0)]);
        let validator = OutlierSigmaValidator { sigma: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(
            report
                .details
                .unwrap()
                .contains("standard deviation is zero")
        );
    }

    #[test]
    fn skips_when_column_not_numeric() {
        let df = make_str_df(&[Some("a"), Some("b")]);
        let validator = OutlierSigmaValidator { sigma: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("could not be cast"));
    }

    #[test]
    fn skips_when_all_nulls() {
        let df = make_f64_df(&[None, None, None]);
        let validator = OutlierSigmaValidator { sigma: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(
            report
                .details
                .unwrap()
                .contains("mean or standard deviation")
        );
    }

    #[test]
    fn skips_when_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<f64>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = OutlierSigmaValidator { sigma: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(
            report
                .details
                .unwrap()
                .contains("mean or standard deviation")
        );
    }
}
