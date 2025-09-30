pub mod engine;
pub mod logging;
pub mod contracts;
pub mod connectors;
pub mod drivers;
pub mod traits;
pub mod validators;
pub mod movement;
pub mod runner;
pub mod profiles;


// Optional: re-export top-level functions/types for convenience
pub use engine::{
    run_contract_validation,
    run_health_check,
    ValidationOutcome,
    HealthStatus,
    execute_validation,
    validate_dataframe,
    run_health_command,
    list_profiles,
    test_profile,
    ProfileList,
    ProfileTestResult,
    list_contracts,
    get_contract,
    validate_contract,
    ContractList,
    ContractInfo,
    ContractValidation,
    verify_logs,
    LogVerification,
    log_action,
};

