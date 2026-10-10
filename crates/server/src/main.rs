mod app;
mod auth;
mod error;
mod http;
mod route;
mod sync;

use app::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required")?;
    let address: std::net::SocketAddr = std::env::var("ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3001".into())
        .parse()?;
    let app = AppState::connect(&database_url)
        .await
        .map_err(|_| "Could not connect to PostgreSQL")?;
    if std::env::args().nth(1).as_deref() == Some("create-pharmacy") {
        let username = std::env::var("PHARMACY_BOOTSTRAP_USERNAME")?;
        let password = std::env::var("PHARMACY_BOOTSTRAP_PASSWORD")?;
        app.bootstrap_pharmacy(&username, &password).await?;
        println!("Pharmacy account created.");
        return Ok(());
    }
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "pharmacy server listening");
    axum::serve(listener, route::router(app)).await?;
    Ok(())
}
