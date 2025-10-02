// src/validators/column/type_validator.rs

use crate::validators::{ValidationReport, ValidationResult, Validator};
use polars::prelude::*;

pub struct TypeValidator {
    pub dtype: String,
}

impl Validator for TypeValidator {
    fn name(&self) -> &'static str {
        "Type"
    }

    fn validate(&self, df: &DataFrame, column_name: &str) -> ValidationResult<ValidationReport> {
        let series = df.column(column_name)?;

        let actual_dtype = format!("{:?}", series.dtype());

        if actual_dtype == self.dtype {
            Ok(ValidationReport {
                status: "pass",
                details: None,
            })
        } else {
            Ok(ValidationReport {
                status: "fail",
                details: Some(format!("expected={}, actual={}", self.dtype, actual_dtype)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_i64_df(values: &[i64]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    fn make_f64_df(values: &[f64]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    fn make_str_df(values: &[&str]) -> DataFrame {
        let s = Series::new("col".into(), values.to_vec());
        DataFrame::new(vec![s.into()]).unwrap()
    }

    #[test]
    fn passes_when_dtype_matches() {
        let df = make_i64_df(&[1, 2, 3]);
        let validator = TypeValidator { dtype: "Int64".to_string() };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_when_dtype_does_not_match() {
        let df = make_i64_df(&[1, 2, 3]);
        let validator = TypeValidator { dtype: "Float64".to_string() };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
        assert!(report.details.unwrap().contains("expected=Float64"));
    }

    #[test]
    fn works_with_float_column() {
        let df = make_f64_df(&[1.0, 2.0, 3.0]);
        let validator = TypeValidator { dtype: "Float64".to_string() };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn works_with_string_column() {
        let df = make_str_df(&["a", "b", "c"]);
        let validator = TypeValidator { dtype: "String".to_string() };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "pass");
    }

    #[test]
    fn fails_on_empty_column_with_wrong_dtype() {
        let s: Series = Series::new("col".into(), Vec::<i64>::new());
        let df = DataFrame::new(vec![s.into()]).unwrap();
        let validator = TypeValidator { dtype: "Float64".to_string() };
        let report = validator.validate(&df, "col").unwrap();
        assert_eq!(report.status, "fail");
    }
}
