use crate::{
    app::AppState,
    auth::middleware,
    http::{api, site},
};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

pub fn router(app: AppState) -> Router {
    let browser_routes = Router::new()
        .route("/", get(site::dashboard))
        .route("/logout", post(site::logout))
        .route_layer(from_fn_with_state(
            app.clone(),
            middleware::require_browser_pharmacy,
        ))
        .with_state(app.clone());

    let protected_api = Router::new()
        .route("/api/v1/auth/me", get(api::me))
        .route("/api/v1/auth/logout", post(api::logout))
        .route("/api/v1/medicines", get(api::full_medicines))
        .route("/api/v1/medicines/changes", get(api::medicine_changes))
        .route("/api/v1/sync", post(api::synchronize))
        .route_layer(from_fn_with_state(
            app.clone(),
            middleware::require_api_pharmacy,
        ))
        .with_state(app.clone());

    Router::new()
        .route("/health", get(api::health))
        .route("/login", get(site::login_page).post(site::login))
        .route("/api/v1/auth/login", post(api::login))
        .merge(browser_routes)
        .merge(protected_api)
        .layer(TraceLayer::new_for_http())
        .with_state(app)
}
