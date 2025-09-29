use crate::contracts::{ContractType, SchemaContracts};
use crate::error::ValidationResult;
use crate::logging::schema::RuleResult;
use crate::validators::column::{
    BooleanValidator, CompletenessValidator, DateFormatValidator, DistinctnessValidator,
    InSetValidator, MaxLengthValidator, MeanBetweenValidator, NotInSetValidator, NotNullValidator,
    OutlierSigmaValidator, PatternValidator, RangeValidator, StdevBetweenValidator, TypeValidator,
    UniqueValidator,
};
use crate::validators::compound::CompoundUniqueValidator;
use crate::validators::file::{FileCompletenessValidator, RowCountValidator};
use crate::validators::{CompoundValidator, FileValidator, Validator};
use polars::prelude::*;

pub fn validate_dataframe(
    df: &DataFrame,
    contracts: &SchemaContracts,
) -> ValidationResult<Vec<RuleResult>> {
    let mut results: Vec<RuleResult> = Vec::new();

    // --- File-Level Validation ---
    if let Some(file_contracts) = &contracts.file {
        for contract_rule in &file_contracts.validation {
            let validator: Box<dyn FileValidator> = match contract_rule {
                ContractType::RowCount { min, max } => Box::new(RowCountValidator {
                    min: *min,
                    max: *max,
                }),
                ContractType::Completeness { min_ratio } => Box::new(FileCompletenessValidator {
                    min_ratio: *min_ratio,
                }),
                _ => continue,
            };
            let report = validator.validate(df)?;
            results.push(RuleResult {
                column: "file".to_string(),
                rule: validator.name().to_string(),
                result: report.status.to_string(),
                details: report.details.clone(),
            });
        }
    }

    // --- Column-Level Validation ---
    for col in &contracts.columns {
        for contract_rule in &col.validation {
            let validator: Box<dyn Validator> = match contract_rule {
                ContractType::NotNull => Box::new(NotNullValidator),
                ContractType::Unique => Box::new(UniqueValidator),
                ContractType::Boolean => Box::new(BooleanValidator),
                ContractType::Range { min, max } => Box::new(RangeValidator {
                    min: *min,
                    max: *max,
                }),
                ContractType::Pattern { pattern } => Box::new(PatternValidator {
                    pattern: pattern.clone(),
                }),
                ContractType::MaxLength { value } => Box::new(MaxLengthValidator { value: *value }),
                ContractType::MeanBetween { min, max } => Box::new(MeanBetweenValidator {
                    min: *min,
                    max: *max,
                }),
                ContractType::StdevBetween { min, max } => Box::new(StdevBetweenValidator {
                    min: *min,
                    max: *max,
                }),
                ContractType::Completeness { min_ratio } => Box::new(CompletenessValidator {
                    min_ratio: *min_ratio,
                }),
                ContractType::InSet { values } => Box::new(InSetValidator {
                    values: values.iter().cloned().collect(),
                }),
                ContractType::NotInSet { values } => Box::new(NotInSetValidator {
                    values: values.iter().cloned().collect(),
                }),
                ContractType::Type { dtype } => Box::new(TypeValidator {
                    dtype: dtype.clone(),
                }),
                ContractType::OutlierSigma { sigma } => {
                    Box::new(OutlierSigmaValidator { sigma: *sigma })
                }
                ContractType::DateFormat { format } => Box::new(DateFormatValidator {
                    format: format.clone(),
                }),
                ContractType::Distinctness { min_ratio } => Box::new(DistinctnessValidator {
                    min_ratio: *min_ratio,
                }),
                _ => continue,
            };

            let report = validator.validate(df, &col.name)?;
            results.push(RuleResult {
                column: col.name.clone(),
                rule: validator.name().to_string(),
                result: report.status.to_string(),
                details: report.details.clone(),
            });
        }
    }

    // --- Compound-Level Validation ---
    if let Some(compounds) = &contracts.compound_unique {
        for cu in compounds {
            let validator: Box<dyn CompoundValidator> = Box::new(CompoundUniqueValidator {
                columns: cu.columns.clone(),
            });
            let report = validator.validate(df)?;
            results.push(RuleResult {
                column: "compound".to_string(),
                rule: validator.name().to_string(),
                result: report.status.to_string(),
                details: report.details.clone(),
            });
        }
    }

    Ok(results)
}
