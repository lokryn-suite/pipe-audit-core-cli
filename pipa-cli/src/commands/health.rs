use pipa::health::run_health_check;

pub async fn run() {
    let (_status, message) = run_health_check(true); // pass true to log to console
    println!("{}", message);
}
