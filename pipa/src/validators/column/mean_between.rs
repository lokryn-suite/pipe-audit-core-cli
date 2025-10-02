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
    fn passes_when_mean_within_range() {
        let df = make_f64_df(&[Some(5.0), Some(7.0), Some(9.0)]); // mean = 7.0
        let validator = MeanBetweenValidator { min: 6.0, max: 8.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_mean_below_min() {
        let df = make_f64_df(&[Some(1.0), Some(2.0), Some(3.0)]); // mean = 2.0
        let validator = MeanBetweenValidator {
            min: 3.5,
            max: 10.0,
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("observed_mean=2.00"));
    }

    #[test]
    fn fails_when_mean_above_max() {
        let df = make_f64_df(&[Some(10.0), Some(20.0), Some(30.0)]); // mean = 20.0
        let validator = MeanBetweenValidator {
            min: 0.0,
            max: 15.0,
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("observed_mean=20.00"));
    }

    #[test]
    fn skips_when_column_not_numeric() {
        let df = make_str_df(&[Some("a"), Some("b")]);
        let validator = MeanBetweenValidator { min: 0.0, max: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("could not be cast"));
    }

    #[test]
    fn skips_when_all_nulls() {
        let df = make_f64_df(&[None, None, None]);
        let validator = MeanBetweenValidator { min: 0.0, max: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("no non-null values"));
    }

    #[test]
    fn skips_when_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<f64>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = MeanBetweenValidator { min: 0.0, max: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("no non-null values"));
    }
}
