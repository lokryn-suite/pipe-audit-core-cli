// src/validators/column/in_set.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;
use std::collections::HashSet;

pub struct InSetValidator {
    pub values: HashSet<String>,
}

impl Validator for InSetValidator {
    fn name(&self) -> &'static str {
        "InSet"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if !series.dtype().is_string() {
            return Ok(ValidationReport {
                status: "skipped",
                details: Some("column is not a string type".to_string()),
            });
        }

        let allowed_values: Vec<String> = self.values.iter().cloned().collect();

        // Use the working pattern from our test with lazy DataFrame operations
        let result = df
            .clone()
            .lazy()
            .select([col(column_name)
                .is_in(
                    lit(Series::new("allowed".into(), allowed_values)).implode(),
                    false,
                )
                .not()])
            .collect()?;

        let bad_series = result.column(column_name)?;
        let bad_count: u32 = bad_series.bool()?.sum().unwrap_or(0);

        if bad_count > 0 {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("bad_count={}", bad_count)),
            })
        } else {
            Ok(ValidationReport {
                status: "pass",
                details: None,
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

    fn make_int_df(values: &[i32]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    fn make_validator(allowed: &[&str]) -> InSetValidator {
        InSetValidator {
            values: allowed.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn passes_when_all_values_in_set() {
        let df = make_str_df(&[Some("a"), Some("b"), Some("c")]);
        let validator = make_validator(&["a", "b", "c"]);
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_some_values_not_in_set() {
        let df = make_str_df(&[Some("a"), Some("x"), Some("b"), Some("y")]);
        let validator = make_validator(&["a", "b"]);
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        // 2 values ("x", "y") not in set
        assert!(report.details.unwrap().contains("bad_count=2"));
    }

    #[test]
    fn ignores_null_values() {
        let df = make_str_df(&[Some("a"), None, Some("b")]);
        let validator = make_validator(&["a", "b"]);
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = make_validator(&["a", "b"]);
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn skips_on_non_string_column() {
        let df = make_int_df(&[1, 2, 3]);
        let validator = make_validator(&["1", "2"]);
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("not a string"));
    }
}
