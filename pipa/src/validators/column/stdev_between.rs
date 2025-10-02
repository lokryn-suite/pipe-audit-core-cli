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
    fn passes_when_stdev_within_range() {
        // values: [1, 2, 3] → stdev ≈ 1.0
        let df = make_f64_df(&[Some(1.0), Some(2.0), Some(3.0)]);
        let validator = StdevBetweenValidator { min: 0.5, max: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_when_stdev_below_min() {
        // values: [5, 5, 5] → stdev = 0.0
        let df = make_f64_df(&[Some(5.0), Some(5.0), Some(5.0)]);
        let validator = StdevBetweenValidator { min: 0.1, max: 2.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("observed_stdev=0.00"));
    }

    #[test]
    fn fails_when_stdev_above_max() {
        // values: [1, 100] → stdev large
        let df = make_f64_df(&[Some(1.0), Some(100.0)]);
        let validator = StdevBetweenValidator { min: 0.0, max: 10.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("observed_stdev"));
    }

    #[test]
    fn skips_when_column_not_numeric() {
        let df = make_str_df(&[Some("a"), Some("b")]);
        let validator = StdevBetweenValidator { min: 0.0, max: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("could not be cast"));
    }

    #[test]
    fn skips_when_all_nulls() {
        let df = make_f64_df(&[None, None, None]);
        let validator = StdevBetweenValidator { min: 0.0, max: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("could not be calculated"));
    }

    #[test]
    fn skips_when_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<f64>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = StdevBetweenValidator { min: 0.0, max: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("could not be calculated"));
    }
}

