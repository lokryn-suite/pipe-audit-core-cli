use pipa::api::server;
use pipa::logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    logging::init_logging();

    server::run().await
}
