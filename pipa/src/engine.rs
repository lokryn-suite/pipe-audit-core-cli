pub mod contracts;
pub mod init;
pub mod logging;
pub mod logs;
pub mod profiles;
pub mod system;
pub mod validation;

// Re-export core functions for public use

pub use self::contracts::{
    ContractInfo, ContractList, ContractValidation, get_contract, list_contracts, validate_contract,
};
pub use self::logging::log_action;
pub use self::logs::{LogVerification, verify_logs};
pub use self::profiles::{ProfileList, ProfileTestResult, list_profiles, test_profile};
pub use self::system::{check_system_health, run_health_check};
pub use self::validation::{execute_validation, validate_dataframe};
