use crate::api::routes;
use std::net::SocketAddr;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PipeAudit API Server starting...");

    let app = routes::create_router();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("📡 Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
