// src/validators/column/boolean.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use once_cell::sync::Lazy;
use polars::prelude::*;
use std::collections::HashSet;

// Using Lazy from once_cell to create a static HashSet for efficiency.
static ALLOWED_BOOLEAN_VALUES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["true", "false", "1", "0", "t", "f", "yes", "no", "y", "n"]
        .iter()
        .cloned()
        .collect()
});

pub struct BooleanValidator;

impl Validator for BooleanValidator {
    fn name(&self) -> &'static str {
        "Boolean"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if let Ok(utf8_chunked) = series.str() {
            let bad_count = utf8_chunked
                .into_iter()
                .filter(|opt_val| {
                    if let Some(val) = opt_val {
                        !ALLOWED_BOOLEAN_VALUES.contains(val.to_lowercase().as_str())
                    } else {
                        false // Null values are not considered non-boolean
                    }
                })
                .count();

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
        } else {
            Ok(ValidationReport {
                status: "skipped",
                details: Some("column is not a string type".to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_df(values: &[Option<&str>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }
    #[test]
    fn passes_on_valid_booleans() {
        let df = make_df(&[Some("true"), Some("FALSE"), Some("1"), Some("n"), None]);
        let validator = BooleanValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_on_invalid_values() {
        let df = make_df(&[Some("maybe"), Some("true"), Some("nope")]);
        let validator = BooleanValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert_eq!(report.details, Some("bad_count=2".to_string()));
    }
    #[test]
    fn skips_on_non_string_column() {
        let s = Series::new("col".into(), &[1, 0, 1]);
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = BooleanValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("not a string"));
    }
}
