// src/validators/column/range.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct RangeValidator {
    pub min: i64,
    pub max: i64,
}

impl Validator for RangeValidator {
    fn name(&self) -> &'static str {
        "Range"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if let Ok(values) = series.i64() {
            // Create a boolean mask for values outside the desired range.
            let mask = values.lt(self.min) | values.gt(self.max);
            let bad_count = mask.sum().unwrap_or(0);

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!(
                        "bad_count={}, min={}, max={}",
                        bad_count, self.min, self.max
                    )),
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
                details: Some("column is not an integer type".to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_i64_df(values: &[Option<i64>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    fn make_str_df(values: &[Option<&str>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    #[test]
    fn passes_when_all_values_within_range() {
        let df = make_i64_df(&[Some(5), Some(10), Some(15)]);
        let validator = RangeValidator { min: 0, max: 20 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_values_below_min() {
        let df = make_i64_df(&[Some(-5), Some(10), Some(15)]);
        let validator = RangeValidator { min: 0, max: 20 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("bad_count=1"));
    }

    #[test]
    fn fails_when_values_above_max() {
        let df = make_i64_df(&[Some(5), Some(25), Some(15)]);
        let validator = RangeValidator { min: 0, max: 20 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("bad_count=1"));
    }

    #[test]
    fn fails_when_all_values_out_of_range() {
        let df = make_i64_df(&[Some(-10), Some(30)]);
        let validator = RangeValidator { min: 0, max: 20 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("bad_count=2"));
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<i64>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = RangeValidator { min: 0, max: 20 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn skips_on_non_integer_column() {
        let df = make_str_df(&[Some("a"), Some("b")]);
        let validator = RangeValidator { min: 0, max: 20 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("not an integer"));
    }
}
