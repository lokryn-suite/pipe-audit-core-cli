use crate::contracts::{ContractType, SchemaContracts};
use crate::drivers::get_driver;
use crate::error::ValidationResult;
use crate::logging::schema::{AuditLogEntry, Contract, Executor, RuleResult};
use crate::logging::writer::log_event;
use crate::validators::column::{
    BooleanValidator, CompletenessValidator, DateFormatValidator, DistinctnessValidator,
    InSetValidator, MaxLengthValidator, MeanBetweenValidator, NotInSetValidator, NotNullValidator,
    OutlierSigmaValidator, PatternValidator, RangeValidator, StdevBetweenValidator, TypeValidator,
    UniqueValidator,
};
use crate::validators::compound::CompoundUniqueValidator;
use crate::validators::file::{FileCompletenessValidator, RowCountValidator};
use crate::validators::{CompoundValidator, FileValidator, Validator};
use anyhow::Context;
use chrono::Utc;
use polars::prelude::*;

/// Core validation orchestration - audit logging only, no console output
pub async fn execute_validation(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
    executor: &Executor,
) -> ValidationResult<Vec<RuleResult>> {
    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "validation_start",
        contract: Some(Contract {
            name: &contracts.contract.name,
            version: &contracts.contract.version,
        }),
        target: None,
        results: None,
        executor: executor.clone(),
        details: Some(&format!("bytes={}, extension={}", data.len(), extension)),
        summary: None,
    });

    let driver =
        get_driver(extension).context("Failed to find a suitable driver for the extension")?;

    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "driver_found",
        contract: None,
        target: None,
        results: None,
        executor: executor.clone(),
        details: Some(&format!("extension={}", extension)),
        summary: None,
    });

    let df = driver
        .load(data)
        .context("Failed to parse data from memory")?;

    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "dataframe_parsed",
        contract: None,
        target: None,
        results: None,
        executor: executor.clone(),
        details: Some(&format!("rows={}, cols={}", df.height(), df.width())),
        summary: None,
    });

    let results: Vec<RuleResult> = validate_dataframe(&df, contracts)?;

    log_event(&AuditLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: "AUDIT",
        event: "validation_summary",
        contract: Some(Contract {
            name: &contracts.contract.name,
            version: &contracts.contract.version,
        }),
        target: None,
        results: Some(results.clone()),
        executor: executor.clone(),
        details: None,
        summary: None,
    });

    Ok(results)
}

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
