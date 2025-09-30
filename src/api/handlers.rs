use crate::engine::{
    run_contract_validation,
    run_health_check as engine_run_health_check,
    list_contracts as engine_list_contracts,
    get_contract as engine_get_contract,
    validate_contract as engine_validate_contract,
    list_profiles as engine_list_profiles,
    test_profile as engine_test_profile,
    verify_logs as engine_verify_logs
};
use crate::logging::schema::Executor;
use crate::logging::verify::FileStatus;
use axum::{extract::Path, http::StatusCode, Json, extract::Query};
use glob;
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

    let (status, _message) = engine_run_health_check(&executor, false);
    
    (
        StatusCode::OK,
        Json(HealthResponse {
            healthy: status.healthy,
            version: crate::VERSION.to_string(),
        }),
    )
}
// ===== RUN VALIDATION =====

pub async fn run_contract(Path(contract_name): Path<String>) -> (StatusCode, Json<Value>) {
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
        Ok((outcome, message)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": message,
                "outcome": {
                    "passed": outcome.passed,
                    "pass_count": outcome.pass_count,
                    "fail_count": outcome.fail_count
                }
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": "Validation failed"
            })),
        ),
    }
}

pub async fn run_all() -> (StatusCode, Json<Value>) {
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
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": "Failed to read contracts directory"
            })),
        ),
    };

    let mut results = Vec::new();
    let mut all_passed = true;

    // Run each contract
    for path in contract_files {
        let contract_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };
        
        match run_contract_validation(contract_name, &executor, false).await {
            Ok((outcome, message)) => {
                results.push(json!({
                    "contract": contract_name,
                    "success": true,
                    "message": message,
                    "outcome": {
                        "passed": outcome.passed,
                        "pass_count": outcome.pass_count,
                        "fail_count": outcome.fail_count
                    }
                }));
                if !outcome.passed {
                    all_passed = false;
                }
            }
            Err(_) => {
                results.push(json!({
                    "contract": contract_name,
                    "success": false,
                    "message": "Validation failed"
                }));
                all_passed = false;
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": all_passed,
            "message": if all_passed { "All contracts validated successfully" } else { "Some contracts failed validation" },
            "results": results
        })),
    )
}
pub async fn list_contracts() -> (StatusCode, Json<Value>) {
    match engine_list_contracts() {
        Ok((contract_list, message)) => (
            StatusCode::OK,
            Json(json!({
                "contracts": contract_list.contracts,
                "message": message
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to read contracts"
            })),
        ),
    }
}

pub async fn get_contract(Path(name): Path<String>) -> (StatusCode, Json<Value>) {
    let (info, message) = engine_get_contract(&name);
    (
        StatusCode::OK,
        Json(json!({
            "name": info.name,
            "version": info.version,
            "exists": info.exists,
            "message": message
        })),
    )
}

#[derive(Deserialize)]
pub struct ValidateContractRequest {
    pub content: String,
}

// GET /api/v1/contracts/:name/validate
pub async fn validate_contract(Path(name): Path<String>) -> (StatusCode, Json<Value>) {
    let (validation, message) = engine_validate_contract(&name);

    if validation.valid {
        (
            StatusCode::OK,
            Json(json!({
                "valid": true,
                "message": message
            })),
        )
    } else {
        (
            StatusCode::OK,
            Json(json!({
                "valid": false,
                "error": validation.error,
                "message": message
            })),
        )
    }
}
pub async fn list_profiles() -> (StatusCode, Json<Value>) {
    match engine_list_profiles() {
        Ok((profile_list, message)) => (
            StatusCode::OK,
            Json(json!({
                "profiles": profile_list.profiles,
                "message": message
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to read profiles"
            })),
        ),
    }
}

#[derive(Deserialize)]
pub struct TestProfileRequest {
    pub profile: String,
}

pub async fn test_profile(Json(payload): Json<TestProfileRequest>) -> (StatusCode, Json<Value>) {
    let (result, message) = engine_test_profile(&payload.profile).await;

    if result.exists && result.connected {
        (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": message
            })),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": message,
                "exists": result.exists,
                "connected": result.connected
            })),
        )
    }
}

pub async fn verify_logs(Query(params): Query<VerifyLogsQuery>) -> (StatusCode, Json<Value>) {
    let (verification, message) = engine_verify_logs(params.date.as_deref());

    (
        StatusCode::OK,
        Json(json!({
            "valid": verification.valid,
            "verified": verification.verified,
            "mismatched": verification.mismatched,
            "missing": verification.missing,
            "malformed": verification.malformed,
            "unsealed": verification.unsealed,
            "message": message,
            "files": verification.files.iter().map(|f| json!({
                "filename": f.filename,
                "status": match f.status {
                    FileStatus::Verified => "verified",
                    FileStatus::Mismatched => "mismatched",
                    FileStatus::Missing => "missing",
                    FileStatus::Malformed => "malformed",
                    FileStatus::Unsealed => "unsealed",
                }
            })).collect::<Vec<_>>()
        })),
    )
}