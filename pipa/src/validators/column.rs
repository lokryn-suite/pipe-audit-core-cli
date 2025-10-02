//! Column-level validators.
//!
//! This module declares and re-exports all individual column validators.
//! Each validator implements the `Validator` trait defined in
//! `validators.rs`, and enforces a specific rule on a single column
//! of a Polars `DataFrame`.
//!
//! The engine (`engine/validation.rs`) imports these re-exports and
//! dispatches to them based on the `ContractType` enum.
//!
//! ## Adding a new column validator
//! 1. Create a new module file in this directory (e.g. `my_rule.rs`).
//! 2. Implement the `Validator` trait for your struct.
//! 3. Add a `pub mod my_rule;` declaration here.
//! 4. Add a `pub use my_rule::MyRuleValidator;` re-export here.
//! 5. Add a new `ContractType::MyRule` variant in `contracts/types.rs`.
//! 6. Wire it into the `match` in `engine/validation.rs`.
//!
//! This keeps the system modular and contributor-friendly.


// -----------------------------------------------------------------------------
// Declare all individual column validator modules
// -----------------------------------------------------------------------------
pub mod boolean;
pub mod completeness;
pub mod date_format;
pub mod distinctness;
pub mod in_set;
pub mod max_length;
pub mod mean_between;
pub mod not_in_set;
pub mod not_null;
pub mod outlier_sigma;
pub mod pattern;
pub mod range;
pub mod stdev_between;
pub mod type_validator;
pub mod unique;

// -----------------------------------------------------------------------------
// Re-export each validator struct for easy access from the engine
// -----------------------------------------------------------------------------
pub use boolean::BooleanValidator;
pub use completeness::CompletenessValidator;
pub use date_format::DateFormatValidator;
pub use distinctness::DistinctnessValidator;
pub use in_set::InSetValidator;
pub use max_length::MaxLengthValidator;
pub use mean_between::MeanBetweenValidator;
pub use not_in_set::NotInSetValidator;
pub use not_null::NotNullValidator;
pub use outlier_sigma::OutlierSigmaValidator;
pub use pattern::PatternValidator;
pub use range::RangeValidator;
pub use stdev_between::StdevBetweenValidator;
pub use type_validator::TypeValidator;
pub use unique::UniqueValidator;
