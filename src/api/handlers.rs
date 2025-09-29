use crate::core::orchestration::{run_contract_validation, check_system_health};
use crate::logging::schema::Executor;
use axum::{extract::Path, http::StatusCode, Json};
use hostname;
use serde::Serialize;
use whoami;

// ===== REQUEST/RESPONSE TYPES =====

#[derive(Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
}

// ===== HEALTH =====


pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    let status = crate::core::orchestration::run_health_check(&executor, false);
    
    (
        StatusCode::OK,
        Json(HealthResponse {
            healthy: status.healthy,
            version: crate::VERSION.to_string(),
        }),
    )
}
// ===== RUN VALIDATION =====

pub async fn run_contract(Path(contract_name): Path<String>) -> StatusCode {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    // Use orchestration layer - no console output for API
    match run_contract_validation(&contract_name, &executor, false).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
