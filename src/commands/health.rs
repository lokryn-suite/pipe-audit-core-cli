use crate::core::orchestration::run_health_check;
use crate::logging::schema::Executor;
use hostname;
use whoami;

pub async fn run() {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    run_health_check(&executor, true);
}