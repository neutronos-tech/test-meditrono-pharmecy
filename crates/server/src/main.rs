mod app;
mod auth;
mod error;
mod http;
mod licence;
mod route;
mod sync;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use app::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required")?;
    let address: std::net::SocketAddr = std::env::var("ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3001".into())
        .parse()?;
    #[cfg_attr(not(debug_assertions), allow(unused_mut))]
    let mut app = AppState::connect(&database_url)
        .await
        .map_err(|_| "Could not connect to PostgreSQL")?;
    if let Ok(key) = std::env::var(licence::KEY_ENV) {
        app.licence_key = Some(std::sync::Arc::new(licence::LicenceKey::parse(&key)?));
    } else {
        tracing::warn!(
            "{} is not set; licence renewal is disabled",
            licence::KEY_ENV
        );
    }
    #[cfg(debug_assertions)]
    if let Ok(token) = std::env::var(contracts::auth::dev::TOKEN_ENV) {
        app.enable_dev_token(&token).await?;
    }
    if std::env::args().nth(1).as_deref() == Some("create-pharmacy") {
        let username = std::env::var("PHARMACY_BOOTSTRAP_USERNAME")?;
        let password = std::env::var("PHARMACY_BOOTSTRAP_PASSWORD")?;
        app.bootstrap_pharmacy(&username, &password).await?;
        println!("Pharmacy account created.");
        return Ok(());
    }
    if std::env::args().nth(1).as_deref() == Some("extend-device") {
        let mut args = std::env::args().skip(2);
        let (Some(device_id), Some(days)) = (args.next(), args.next()) else {
            return Err("usage: server extend-device <device-id> <days>".into());
        };
        let paid_until = app.extend_device(device_id.parse()?, days.parse()?).await?;
        println!("Device {device_id} is paid until {paid_until}.");
        return Ok(());
    }
    if std::env::args().nth(1).as_deref() == Some("new-licence-key") {
        println!("{}", licence::LicenceKey::generate_env_value(1));
        return Ok(());
    }
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "pharmacy server listening");
    axum::serve(listener, route::router(app)).await?;
    Ok(())
}
