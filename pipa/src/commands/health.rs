use pipa::health::run_health_check;
use pipa::logging::JsonlLogger;

/// Run a system health check.
///
/// Delegates to `pipa::health::run_health_check(true)`, which performs
/// environment and connector checks. The `true` flag indicates that
/// results should also be logged to the console.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa health
/// ```
pub async fn run() {
    // Run the health check; ignore the status object here, just print the message
    let logger = JsonlLogger::default();
    let (_status, message) = run_health_check(&logger, true);
    println!("{}", message);
}
