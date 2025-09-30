use crate::core::orchestration::{run_contract_validation};
use crate::logging::schema::Executor;
use axum::{extract::Path, http::StatusCode, Json, extract::Query};
use hostname;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use whoami;

// ===== REQUEST/RESPONSE TYPES =====

#[derive(Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
}

#[derive(Deserialize)]
pub struct VerifyLogsQuery {
    pub date: Option<String>,
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

pub async fn run_all() -> StatusCode {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    // Get all contract files
    let contract_files: Vec<_> = match glob::glob("contracts/*.toml") {
        Ok(paths) => paths.filter_map(Result::ok).collect(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Run each contract
    for path in contract_files {
        let contract_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };
        
        if let Err(_) = run_contract_validation(contract_name, &executor, false).await {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::NO_CONTENT
}
pub async fn list_contracts() -> (StatusCode, Json<Value>) {
    let contracts: Vec<String> = match glob::glob("contracts/*.toml") {
        Ok(paths) => paths
            .filter_map(Result::ok)
            .filter_map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))),
    };

    (StatusCode::OK, Json(json!({ "contracts": contracts })))
}

pub async fn get_contract(Path(name): Path<String>) -> (StatusCode, Json<Value>) {
    use crate::contracts::load_contract_for_file;
    use std::path::Path as StdPath;

    let contract_path = format!("contracts/{}.toml", name);
    
    if !StdPath::new(&contract_path).exists() {
        return (
            StatusCode::OK,
            Json(json!({
                "name": name,
                "exists": false
            })),
        );
    }

    let contract = load_contract_for_file(StdPath::new(&contract_path));
    (
        StatusCode::OK,
        Json(json!({
            "name": contract.contract.name,
            "version": contract.contract.version,
            "exists": true
        })),
    )
}

#[derive(Deserialize)]
pub struct ValidateContractRequest {
    pub content: String,
}

// GET /api/v1/contracts/:name/validate
pub async fn validate_contract(Path(name): Path<String>) -> (StatusCode, Json<Value>) {
    use crate::contracts::load_contract_for_file;
    use std::path::Path as StdPath;

    let contract_path = format!("contracts/{}.toml", name);
    
    if !StdPath::new(&contract_path).exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "valid": false,
                "error": "Contract not found"
            })),
        );
    }

    // Try to load it - if it loads, it's valid
    match std::panic::catch_unwind(|| load_contract_for_file(StdPath::new(&contract_path))) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "valid": true })),
        ),
        Err(_) => (
            StatusCode::OK,
            Json(json!({
                "valid": false,
                "error": "Contract failed to parse"
            })),
        ),
    }
}
pub async fn list_profiles() -> (StatusCode, Json<Value>) {
    use crate::profiles::load_profiles;

    match load_profiles() {
        Ok(profiles) => {
            let profile_names: Vec<String> = profiles.keys().cloned().collect();
            (StatusCode::OK, Json(json!({ "profiles": profile_names })))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))),
    }
}

#[derive(Deserialize)]
pub struct TestProfileRequest {
    pub profile: String,
}

pub async fn test_profile(Json(payload): Json<TestProfileRequest>) -> StatusCode {
    use crate::profiles::load_profiles;

    let profiles = match load_profiles() {
        Ok(p) => p,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if profiles.contains_key(&payload.profile) {
        // Profile exists and is configured
        // TODO: Could add actual connectivity test here later
        StatusCode::NO_CONTENT
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn verify_logs(Query(params): Query<VerifyLogsQuery>) -> (StatusCode, Json<Value>) {
    use crate::logging::verify::{verify_date, verify_all};

    let summary = if let Some(date) = params.date {
        verify_date(Some(date.as_str()))
    } else {
        verify_all()
    };

    // VerificationSummary fields: verified, mismatched, missing, malformed, unsealed, files
    let all_valid = summary.mismatched == 0 
        && summary.missing == 0 
        && summary.malformed == 0 
        && summary.unsealed == 0;

    if all_valid {
        (
            StatusCode::OK,
            Json(json!({
                "valid": true,
                "verified": summary.verified,
                "message": "Log integrity verified"
            })),
        )
    } else {
        (
            StatusCode::OK,
            Json(json!({
                "valid": false,
                "verified": summary.verified,
                "mismatched": summary.mismatched,
                "missing": summary.missing,
                "malformed": summary.malformed,
                "unsealed": summary.unsealed,
                "message": "Log integrity check failed"
            })),
        )
    }
}