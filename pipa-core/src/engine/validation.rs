//! Validation orchestration for PipeAudit.
//!
//! This module is the execution heart of the system: it takes a parsed
//! contract (`SchemaContracts`), loads data into a Polars `DataFrame`,
//! applies all file/column/compound rules, and produces structured
//! `RuleResult`s for logging and audit trails.
//!
//! Key responsibilities:
//! - Load data via the appropriate driver (CSV, Parquet, etc.).
//! - Apply file-level, column-level, and compound-level validators.
//! - Emit structured audit log events at each stage.
//! - Return a vector of `RuleResult` for downstream reporting.
//!
//! This module does **not** print to console — it only logs to the audit
//! trail. Console output is handled by higher-level orchestration
//! (`engine/contracts/runner.rs`).

use crate::contracts::{ContractType, SchemaContracts};
use crate::drivers::get_driver;
use crate::logging::error::ValidationResult;
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
use std::collections::HashSet;

/// Execute validation end-to-end against raw data bytes.
///
/// # Arguments
/// * `data` - Raw file contents (CSV, Parquet, etc.).
/// * `extension` - File extension (used to select driver).
/// * `contracts` - Parsed schema contracts to enforce.
/// * `executor` - Metadata about who/where is running validation.
///
/// # Returns
/// * `ValidationResult<Vec<RuleResult>>` - A vector of rule outcomes,
///   or a `ValidationError` if orchestration fails.
///
/// # Logging
/// Emits the following audit events:
/// - `validation_start`
/// - `driver_found`
/// - `dataframe_parsed`
/// - `validation_summary`
pub async fn execute_validation(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
    executor: &Executor,
) -> ValidationResult<Vec<RuleResult>> {
    // --- Start log ---
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

    // --- Driver selection ---
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

    // --- Parse into DataFrame ---
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

    // --- Apply all validators ---
    let results: Vec<RuleResult> = validate_dataframe(&df, contracts)?;

    // --- Summary log ---
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

/// Apply all file-level, column-level, and compound-level validators
/// to a DataFrame according to the provided contracts.
///
/// # Arguments
/// * `df` - Polars DataFrame containing the dataset.
/// * `contracts` - Schema contracts specifying rules.
///
/// # Returns
/// * `ValidationResult<Vec<RuleResult>>` - One `RuleResult` per rule applied.
///
/// # Notes
/// - File-level rules apply to the dataset as a whole.
/// - Column-level rules apply to individual columns.
/// - Compound rules apply across multiple columns.
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
                _ => continue, // skip unsupported rules at file level
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
                    values: values.iter().cloned().collect::<HashSet<String>>(),
                }),
                ContractType::NotInSet { values } => Box::new(NotInSetValidator {
                    values: values.iter().cloned().collect::<HashSet<String>>(),
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
                _ => continue, // skip unsupported rules at column level
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
