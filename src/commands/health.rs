use crate::engine::run_health_command;

pub async fn run() {
    run_health_command().await;
}