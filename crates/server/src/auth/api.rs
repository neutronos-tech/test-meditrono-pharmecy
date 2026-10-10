use crate::{
    app::{AppState, LoginError, SessionKind},
    auth::middleware::ApiCredential,
    error::AppError,
};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use contracts::auth::{PharmacyAccount, PharmacyLoginInput, PharmacyLoginResponse};

pub async fn login(
    State(app): State<AppState>,
    Json(input): Json<PharmacyLoginInput>,
) -> Result<Json<PharmacyLoginResponse>, AppError> {
    let authentication = app
        .login_pharmacy(&input.username, &input.password, SessionKind::Desktop)
        .await
        .map_err(|error| match error {
            LoginError::InvalidCredentials => AppError::Unauthorized,
            LoginError::Database => AppError::Internal,
        })?;
    Ok(Json(PharmacyLoginResponse {
        pharmacy: authentication.pharmacy,
        session_token: authentication.session_token,
    }))
}

pub async fn me(Extension(pharmacy): Extension<PharmacyAccount>) -> Json<PharmacyAccount> {
    Json(pharmacy)
}

pub async fn logout(
    State(app): State<AppState>,
    Extension(credential): Extension<ApiCredential>,
) -> Result<StatusCode, AppError> {
    app.logout_session(&credential.token)
        .await
        .map_err(|_| AppError::Unauthorized)?;
    Ok(StatusCode::NO_CONTENT)
}
