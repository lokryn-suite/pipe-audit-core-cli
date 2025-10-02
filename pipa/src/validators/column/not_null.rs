use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct NotNullValidator;

impl Validator for NotNullValidator {
    fn name(&self) -> &'static str {
        "NotNull"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let null_count = series.null_count();

        if null_count > 0 {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("null_count={}", null_count)),
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

    fn make_int_df(values: &[Option<i32>]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    #[test]
    fn passes_when_no_nulls() {
        let df = make_str_df(&[Some("a"), Some("b"), Some("c")]);
        let validator = NotNullValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
        assert!(report.details.is_none());
    }

    #[test]
    fn fails_when_some_nulls() {
        let df = make_str_df(&[Some("a"), None, Some("b")]);
        let validator = NotNullValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("null_count=1"));
    }

    #[test]
    fn fails_when_all_nulls() {
        let df = make_str_df(&[None, None, None]);
        let validator = NotNullValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("null_count=3"));
    }

    #[test]
    fn passes_on_empty_column() {
        let s: Series = Series::new("col".into(), Vec::<Option<&str>>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = NotNullValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn works_with_integer_column() {
        let df = make_int_df(&[Some(1), None, Some(2), Some(3)]);
        let validator = NotNullValidator;
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("null_count=1"));
    }
}
