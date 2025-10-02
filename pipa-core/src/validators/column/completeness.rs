// src/validators/column/completeness.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct CompletenessValidator {
    pub min_ratio: f64,
}

impl Validator for CompletenessValidator {
    fn name(&self) -> &'static str {
        "Completeness"
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

        let non_null_count = (total_count - series.null_count()) as f64;
        let ratio = non_null_count / total_count as f64;

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
    fn passes_when_all_non_null() {
        let df = make_str_df(&[Some("a"), Some("b"), Some("c")]);
        let validator = CompletenessValidator { min_ratio: 1.0 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn passes_when_ratio_above_threshold() {
        let df = make_str_df(&[Some("a"), None, Some("b")]); // 2/3 non-null = 0.66
        let validator = CompletenessValidator { min_ratio: 0.5 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_when_ratio_below_threshold() {
        let df = make_str_df(&[Some("a"), None, None]); // 1/3 = 0.33
        let validator = CompletenessValidator { min_ratio: 0.75 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("ratio=0.33"));
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = CompletenessValidator { min_ratio: 0.9 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert_eq!(report.details.unwrap(), "column is empty");
    }

    #[test]
    fn fails_when_all_nulls() {
        let df = make_str_df(&[None, None, None]);
        let validator = CompletenessValidator { min_ratio: 0.1 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("ratio=0.00"));
    }

    #[test]
    fn works_with_non_string_column() {
        let df = make_int_df(&[Some(1), None, Some(2)]);
        let validator = CompletenessValidator { min_ratio: 0.5 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }
}
