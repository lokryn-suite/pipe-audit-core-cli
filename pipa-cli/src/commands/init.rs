use pipa::engine::init;

pub async fn run() {
    match init::run() {
        Ok(msg) => println!("✅ {}", msg),
        Err(e) => eprintln!("❌ Init failed: {}", e),
    }
}
