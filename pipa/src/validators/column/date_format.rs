// src/validators/column/date_format.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use chrono::NaiveDateTime;
use polars::prelude::*;

pub struct DateFormatValidator {
    pub format: String,
}

impl Validator for DateFormatValidator {
    fn name(&self) -> &'static str {
        "DateFormat"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        if let Ok(utf8_chunked) = series.str() {
            let bad_count = utf8_chunked
                .into_iter()
                .filter(|opt_val| {
                    if let Some(val) = opt_val {
                        NaiveDateTime::parse_from_str(val, &self.format).is_err()
                    } else {
                        false // Null values don't fail a format match
                    }
                })
                .count();

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!("bad_count={}, format={}", bad_count, self.format)),
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
    fn passes_when_all_values_match_format() {
        let df = make_str_df(&[Some("2024-01-01 12:00:00"), Some("1999-12-31 23:59:59")]);
        let validator = DateFormatValidator {
            format: "%Y-%m-%d %H:%M:%S".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_some_values_do_not_match_format() {
        let df = make_str_df(&[
            Some("2024-01-01 12:00:00"),
            Some("not-a-date"),
            Some("2024/01/01 12:00:00"),
        ]);
        let validator = DateFormatValidator {
            format: "%Y-%m-%d %H:%M:%S".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("bad_count=2"));
    }

    #[test]
    fn ignores_null_values() {
        let df = make_str_df(&[Some("2024-01-01 12:00:00"), None]);
        let validator = DateFormatValidator {
            format: "%Y-%m-%d %H:%M:%S".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = DateFormatValidator {
            format: "%Y-%m-%d %H:%M:%S".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn skips_on_non_string_column() {
        let s = Series::new("col".into(), &[1, 2, 3]);
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = DateFormatValidator {
            format: "%Y-%m-%d %H:%M:%S".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("not a string"));
    }
}
