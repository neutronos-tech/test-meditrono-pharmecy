//! Public plans, self-serve sign-up and the owner's device list.

use crate::{
    app::{AppState, SessionKind},
    error::AppError,
};
use axum::{
    Json,
    extract::{Extension, State},
};
use contracts::{
    auth::{PharmacyAccount, PharmacyLoginResponse},
    saas::{Device, Plan, SignupInput},
};
use sqlx_core::{query::query, row::Row};

fn valid_username(username: &str) -> bool {
    (3..=40).contains(&username.len())
        && username
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

pub async fn plans(State(app): State<AppState>) -> Result<Json<Vec<Plan>>, AppError> {
    let rows = query(
        "SELECT code,name,monthly_price_paisa,yearly_price_paisa,trial_days,ai_scan_cap \
         FROM pharmacy_plans WHERE active ORDER BY is_default DESC, monthly_price_paisa, id",
    )
    .fetch_all(&app.pool)
    .await?;
    let plans = rows
        .iter()
        .map(|row| {
            Ok(Plan {
                code: row.try_get("code")?,
                name: row.try_get("name")?,
                monthly_price_paisa: row.try_get("monthly_price_paisa")?,
                yearly_price_paisa: row.try_get("yearly_price_paisa")?,
                trial_days: row.try_get("trial_days")?,
                ai_scan_cap: row.try_get("ai_scan_cap")?,
            })
        })
        .collect::<Result<_, sqlx_core::error::Error>>()?;
    Ok(Json(plans))
}

pub async fn signup(
    State(app): State<AppState>,
    Json(input): Json<SignupInput>,
) -> Result<Json<PharmacyLoginResponse>, AppError> {
    let username = input.username.trim().to_lowercase();
    if !valid_username(&username) || input.password.len() < 8 || input.password.len() > 200 {
        return Err(AppError::BadRequest);
    }
    let taken: bool =
        query("SELECT EXISTS(SELECT 1 FROM pharmacies WHERE lower(username)=$1) AS taken")
            .bind(&username)
            .fetch_one(&app.pool)
            .await?
            .try_get("taken")?;
    if taken {
        return Err(AppError::Conflict);
    }
    // A concurrent sign-up with the same name loses on the unique index.
    app.bootstrap_pharmacy(&username, &input.password)
        .await
        .map_err(|_| AppError::Conflict)?;
    let authentication = app
        .login_pharmacy(&username, &input.password, SessionKind::Desktop)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(PharmacyLoginResponse {
        pharmacy: authentication.pharmacy,
        session_token: authentication.session_token,
    }))
}

pub async fn devices(
    State(app): State<AppState>,
    Extension(pharmacy): Extension<PharmacyAccount>,
) -> Result<Json<Vec<Device>>, AppError> {
    let rows = query(
        "SELECT id,name,active,paid_until,last_seen_at FROM pharmacy_devices \
         WHERE pharmacy_id=$1 ORDER BY id",
    )
    .bind(pharmacy.id)
    .fetch_all(&app.pool)
    .await?;
    let devices = rows
        .iter()
        .map(|row| {
            Ok(Device {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                active: row.try_get("active")?,
                paid_until: row.try_get("paid_until")?,
                last_seen_at: row.try_get("last_seen_at")?,
            })
        })
        .collect::<Result<_, sqlx_core::error::Error>>()?;
    Ok(Json(devices))
}
