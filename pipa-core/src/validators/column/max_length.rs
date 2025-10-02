// src/validators/column/max_length.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct MaxLengthValidator {
    pub value: usize,
}

impl Validator for MaxLengthValidator {
    fn name(&self) -> &'static str {
        "MaxLength"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if let Ok(ca) = series.str() {
            // Correct method name is str_len_chars
            let lengths = ca.str_len_chars();
            let mask = lengths.gt(self.value as u32);
            let bad_count = mask.sum().unwrap_or(0);

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!(
                        "bad_count={}, max_length={}",
                        bad_count, self.value
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
                details: Some("column is not a string type".to_string()),
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

    #[test]
    fn passes_when_all_values_within_limit() {
        let df = make_str_df(&[Some("a"), Some("bb"), Some("ccc")]);
        let validator = MaxLengthValidator { value: 3 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_values_exceed_limit() {
        let df = make_str_df(&[Some("abcd"), Some("bb"), Some("ccc")]);
        let validator = MaxLengthValidator { value: 3 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("bad_count=1"));
    }

    #[test]
    fn ignores_null_values() {
        let df = make_str_df(&[Some("ok"), None, Some("fine")]);
        let validator = MaxLengthValidator { value: 4 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = MaxLengthValidator { value: 5 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn skips_on_non_string_column() {
        let s = Series::new("col".into(), &[1, 2, 3]);
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = MaxLengthValidator { value: 2 };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("not a string"));
    }
}
