// src/engine.rs

use crate::contracts::{ContractType, SchemaContracts};
use crate::error::ValidationResult;
use crate::logging::log_validation_event;
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

pub fn validate_dataframe(df: &DataFrame, contracts: &SchemaContracts) -> ValidationResult<bool> {
    let contract_name = &contracts.contract.name;
    let contract_version = &contracts.contract.version;
    let mut has_failures = false;

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
            
            if report.status == "fail" {
                has_failures = true;
            }
            
            log_validation_event(
                contract_name,
                contract_version,
                "file",
                validator.name(),
                report.status,
                report.details.as_deref(),
            );
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
            
            if report.status == "fail" {
                has_failures = true;
            }
            
            log_validation_event(
                contract_name,
                contract_version,
                &col.name,
                validator.name(),
                report.status,
                report.details.as_deref(),
            );
        }
    }

    // --- Compound-Level Validation ---
    if let Some(compounds) = &contracts.compound_unique {
        for cu in compounds {
            let validator: Box<dyn CompoundValidator> = Box::new(CompoundUniqueValidator {
                columns: cu.columns.clone(),
            });
            let report = validator.validate(df)?;
            
            if report.status == "fail" {
                has_failures = true;
            }
            
            log_validation_event(
                contract_name,
                contract_version,
                "compound",
                validator.name(),
                report.status,
                report.details.as_deref(),
            );
        }
    }

    Ok(!has_failures) // Return true if no failures, false if any failures
}