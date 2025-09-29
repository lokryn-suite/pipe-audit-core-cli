//! Core business logic shared between CLI and API

pub mod limits;
pub mod orchestration;
pub mod validation;

pub use limits::Limits;
pub use orchestration::{run_contract_validation, run_health_check, ValidationOutcome, HealthStatus};
pub use validation::execute_validation;
