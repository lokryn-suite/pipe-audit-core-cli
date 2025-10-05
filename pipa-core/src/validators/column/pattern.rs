// src/validators/column/pattern.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;
use regex::Regex;

pub struct PatternValidator {
    pub pattern: String,
}

impl Validator for PatternValidator {
    fn name(&self) -> &'static str {
        "Pattern"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let re = Regex::new(&self.pattern)?;

        if let Ok(utf8_chunked) = series.str() {
            let bad_count = utf8_chunked
                .into_iter()
                .filter(|opt_val| {
                    if let Some(val) = opt_val {
                        !re.is_match(val)
                    } else {
                        false // Null values don't fail a pattern match
                    }
                })
                .count();

            if bad_count > 0 {
                Ok(ValidationReport {
                    status: "fail",
                    details: Some(format!("bad_count={}, pattern={}", bad_count, self.pattern)),
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

    fn make_int_df(values: &[i32]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    #[test]
    fn passes_when_all_values_match_pattern() {
        let df = make_str_df(&[Some("abc"), Some("abd"), Some("abe")]);
        let validator = PatternValidator {
            pattern: r"^ab.$".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_some_values_do_not_match_pattern() {
        let df = make_str_df(&[Some("abc"), Some("xyz"), Some("abd")]);
        let validator = PatternValidator {
            pattern: r"^ab.$".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("bad_count=1"));
    }

    #[test]
    fn ignores_null_values() {
        let df = make_str_df(&[Some("abc"), None, Some("abd")]);
        let validator = PatternValidator {
            pattern: r"^ab.$".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = PatternValidator {
            pattern: r"^ab.$".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn skips_on_non_string_column() {
        let df = make_int_df(&[1, 2, 3]);
        let validator = PatternValidator {
            pattern: r"^ab.$".to_string(),
        };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "skipped");
        assert!(report.details.unwrap().contains("not a string"));
    }
}
