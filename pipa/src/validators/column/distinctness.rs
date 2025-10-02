// src/validators/column/distinctness.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct DistinctnessValidator {
    pub min_ratio: f64,
}

impl Validator for DistinctnessValidator {
    fn name(&self) -> &'static str {
        "Distinctness"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let total_count = series.len();
        if total_count == 0 {
            return Ok(ValidationReport {
                status: "pass",
                details: Some("column is empty".to_string()),
            });
        }

        let unique_count = series.n_unique()? as f64;
        let ratio = unique_count / total_count as f64;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_str_df(values: &[Option<&str>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    fn make_int_df(values: &[Option<i32>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    #[test]
    fn passes_when_all_unique() {
        let df = make_str_df(&[Some("a"), Some("b"), Some("c")]);
        let validator = DistinctnessValidator { min_ratio: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn passes_when_ratio_above_threshold() {
        // 3 unique out of 4 total = 0.75
        let df = make_str_df(&[Some("a"), Some("b"), Some("a"), Some("c")]);
        let validator = DistinctnessValidator { min_ratio: 0.7 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_when_ratio_below_threshold() {
        // 2 unique out of 5 total = 0.4
        let df = make_str_df(&[Some("x"), Some("x"), Some("y"), Some("y"), Some("x")]);
        let validator = DistinctnessValidator { min_ratio: 0.6 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("ratio=0.40"));
    }

    #[test]
    fn fails_when_all_values_same() {
        // 1 unique out of 4 total = 0.25
        let df = make_str_df(&[Some("dup"), Some("dup"), Some("dup"), Some("dup")]);
        let validator = DistinctnessValidator { min_ratio: 0.5 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = DistinctnessValidator { min_ratio: 0.9 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert_eq!(report.details.unwrap(), "column is empty");
    }

    #[test]
    fn works_with_integer_column() {
        // 2 unique out of 3 total = 0.66
        let df = make_int_df(&[Some(1), Some(2), Some(1)]);
        let validator = DistinctnessValidator { min_ratio: 0.5 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }
}
