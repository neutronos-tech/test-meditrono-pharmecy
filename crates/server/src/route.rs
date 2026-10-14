use crate::{
    admin,
    app::AppState,
    auth::middleware,
    http::{api, site},
    saas,
};
use axum::http::{HeaderValue, Method, header};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post, put},
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

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
        .route("/api/v1/licence/renew", post(api::renew_licence))
        .route("/api/v1/devices", get(saas::devices))
        .route_layer(from_fn_with_state(
            app.clone(),
            middleware::require_api_pharmacy,
        ))
        .with_state(app.clone());

    let admin_api = Router::new()
        .route("/api/v1/admin/plans", get(admin::plans))
        .route("/api/v1/admin/plans/{code}", put(admin::save_plan))
        .route("/api/v1/admin/customers", get(admin::customers))
        .route(
            "/api/v1/admin/customers/{id}/devices",
            get(admin::customer_devices),
        )
        .route(
            "/api/v1/admin/devices/{id}/extend",
            post(admin::extend_device),
        )
        .route(
            "/api/v1/admin/devices/{id}/active",
            post(admin::set_device_active),
        )
        .route("/api/v1/admin/audit", get(admin::audit_log))
        .route_layer(from_fn_with_state(app.clone(), admin::require_admin))
        .with_state(app.clone());

    Router::new()
        .route("/health", get(api::health))
        .route("/login", get(site::login_page).post(site::login))
        .route("/api/v1/auth/login", post(api::login))
        .route("/api/v1/auth/signup", post(saas::signup))
        .route("/api/v1/plans", get(saas::plans))
        .merge(browser_routes)
        .merge(protected_api)
        .merge(admin_api)
        .layer(cors())
        .layer(TraceLayer::new_for_http())
        .with_state(app)
}

/// The web app runs on another origin (Vercel) and calls the API with a
/// bearer token, so no cookies cross origins. Allowed origins come from
/// `PHARMACY_WEB_ORIGINS` (comma-separated); unset means same-origin only.
fn cors() -> CorsLayer {
    let origins: Vec<HeaderValue> = std::env::var("PHARMACY_WEB_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}
