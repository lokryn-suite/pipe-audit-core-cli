// src/validators/column/unique.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct UniqueValidator;

impl Validator for UniqueValidator {
    fn name(&self) -> &'static str {
        "Unique"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let unique_count = series.n_unique()?;
        let total_count = series.len();

        if unique_count == total_count {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!(
                    "found {} unique values in {} total rows",
                    unique_count, total_count
                )),
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

    #[test]
    fn passes_when_all_values_unique() {
        let df = make_i64_df(&[Some(1), Some(2), Some(3)]);
        let validator = UniqueValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_when_duplicates_present() {
        let df = make_i64_df(&[Some(1), Some(2), Some(1)]);
        let validator = UniqueValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(
            report
                .details
                .unwrap()
                .contains("found 2 unique values in 3 total rows")
        );
    }

    #[test]
    fn fails_when_all_values_same() {
        let df = make_i64_df(&[Some(5), Some(5), Some(5)]);
        let validator = UniqueValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(
            report
                .details
                .unwrap()
                .contains("found 1 unique values in 3 total rows")
        );
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<i64>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = UniqueValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn passes_with_nulls_but_unique_non_nulls() {
        let df = make_i64_df(&[Some(1), Some(2), None]);
        let validator = UniqueValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_with_duplicate_nulls() {
        let df = make_i64_df(&[Some(1), None, None]);
        let validator = UniqueValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(
            report
                .details
                .unwrap()
                .contains("found 2 unique values in 3 total rows")
        );
    }
}
