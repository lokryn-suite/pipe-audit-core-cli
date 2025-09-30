use crate::api::handlers;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_router() -> Router {
    Router::new()
        // Health
        .route("/api/v1/health", get(handlers::health_check))

        // Run validation against data
        .route("/api/v1/run/:contract", post(handlers::run_contract))
        .route("/api/v1/run/all", post(handlers::run_all))  

        // Contract management
        .route("/api/v1/contracts", get(handlers::list_contracts))
        .route("/api/v1/contracts/:name", get(handlers::get_contract))
        .route("/api/v1/contracts/:name/validate", get(handlers::validate_contract))

        // Profiles
        .route("/api/v1/profiles", get(handlers::list_profiles))
        .route("/api/v1/profiles/test", post(handlers::test_profile))

        // Logs - TODO
        .route("/api/v1/logs/verify", get(handlers::verify_logs))
}
