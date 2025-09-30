mod orchestration;
mod validation;
mod health;
mod profiles;
mod contracts;
mod logs;
mod logging;

// Re-export core functions for public use
pub use self::orchestration::{run_contract_validation, run_health_check, ValidationOutcome, HealthStatus};
pub use self::validation::{execute_validation, validate_dataframe};
pub use self::health::run as run_health_command;
pub use self::profiles::{list_profiles, test_profile, ProfileList, ProfileTestResult};
pub use self::contracts::{list_contracts, get_contract, validate_contract, ContractList, ContractInfo, ContractValidation};
pub use self::logs::{verify_logs, LogVerification};
pub use self::logging::log_action;
