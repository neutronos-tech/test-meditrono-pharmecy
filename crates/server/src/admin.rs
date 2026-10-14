//! Founder CMS: edit plans and prices, see customers, activate computers.
//! Guarded by one owner token from `PHARMACY_ADMIN_TOKEN` (at least 32
//! characters), compared by SHA-256 hash in constant time.

use crate::{app::AppState, auth::middleware::bearer_token, error::AppError};
use axum::{
    Json,
    body::Body,
    extract::{Path, Query, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use contracts::saas::{Device, Plan};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx_core::{query::query, row::Row};
use time::OffsetDateTime;
use uuid::Uuid;

pub const TOKEN_ENV: &str = "PHARMACY_ADMIN_TOKEN";

pub fn token_hash(token: &str) -> Result<[u8; 32], String> {
    if token.trim().len() < 32 {
        return Err(format!("{TOKEN_ENV} must be at least 32 characters"));
    }
    Ok(Sha256::digest(token.trim().as_bytes()).into())
}

pub async fn require_admin(
    State(app): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let Some(expected) = app.admin_token_hash else {
        return AppError::NotFound.into_response();
    };
    let given: [u8; 32] = match bearer_token(request.headers()) {
        Some(token) => Sha256::digest(token.as_bytes()).into(),
        None => return AppError::Unauthorized.into_response(),
    };
    // Constant-time comparison of the two hashes.
    if given
        .iter()
        .zip(expected)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        != 0
    {
        return AppError::Unauthorized.into_response();
    }
    next.run(request).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPlan {
    #[serde(flatten)]
    pub plan: Plan,
    pub active: bool,
    pub is_default: bool,
}

pub async fn plans(State(app): State<AppState>) -> Result<Json<Vec<AdminPlan>>, AppError> {
    let rows = query(
        "SELECT code,name,monthly_price_paisa,yearly_price_paisa,trial_days,ai_scan_cap,active,is_default \
         FROM pharmacy_plans ORDER BY id",
    )
    .fetch_all(&app.pool)
    .await?;
    let plans = rows
        .iter()
        .map(|row| {
            Ok(AdminPlan {
                plan: Plan {
                    code: row.try_get("code")?,
                    name: row.try_get("name")?,
                    monthly_price_paisa: row.try_get("monthly_price_paisa")?,
                    yearly_price_paisa: row.try_get("yearly_price_paisa")?,
                    trial_days: row.try_get("trial_days")?,
                    ai_scan_cap: row.try_get("ai_scan_cap")?,
                },
                active: row.try_get("active")?,
                is_default: row.try_get("is_default")?,
            })
        })
        .collect::<Result<_, sqlx_core::error::Error>>()?;
    Ok(Json(plans))
}

/// Creates the plan or updates it by code. Prices are in paisa.
pub async fn save_plan(
    State(app): State<AppState>,
    Path(code): Path<String>,
    Json(input): Json<AdminPlan>,
) -> Result<Json<AdminPlan>, AppError> {
    let plan = &input.plan;
    let valid_code = (2..=40).contains(&code.len())
        && code
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !valid_code
        || plan.name.trim().is_empty()
        || plan.monthly_price_paisa < 0
        || plan.yearly_price_paisa < 0
        || !(0..=365).contains(&plan.trial_days)
        || !(0..=100_000).contains(&plan.ai_scan_cap)
    {
        return Err(AppError::BadRequest);
    }
    let mut tx = app.pool.begin().await?;
    if input.is_default {
        query("UPDATE pharmacy_plans SET is_default=FALSE WHERE is_default AND code<>$1")
            .bind(&code)
            .execute(&mut *tx)
            .await?;
    }
    query(
        "INSERT INTO pharmacy_plans(code,name,monthly_price_paisa,yearly_price_paisa,trial_days,ai_scan_cap,active,is_default) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8) \
         ON CONFLICT(code) DO UPDATE SET name=$2,monthly_price_paisa=$3,yearly_price_paisa=$4,trial_days=$5,ai_scan_cap=$6,active=$7,is_default=$8,updated_at=now()",
    )
    .bind(&code)
    .bind(plan.name.trim())
    .bind(plan.monthly_price_paisa)
    .bind(plan.yearly_price_paisa)
    .bind(plan.trial_days)
    .bind(plan.ai_scan_cap)
    .bind(input.active)
    .bind(input.is_default)
    .execute(&mut *tx)
    .await?;
    let still_default: bool =
        query("SELECT EXISTS(SELECT 1 FROM pharmacy_plans WHERE is_default AND active) AS any")
            .fetch_one(&mut *tx)
            .await?
            .try_get("any")?;
    // New devices need an active default plan to start their trial.
    if !still_default {
        return Err(AppError::BadRequest);
    }
    audit(
        &mut tx,
        None,
        "plan.saved",
        serde_json::to_value(&input).unwrap_or_default(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(input))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub username: String,
    pub active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub devices: i64,
    pub paid_devices: i64,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    /// Keyset cursor: the last customer id of the previous page.
    pub after: Option<Uuid>,
    pub search: Option<String>,
}

pub async fn customers(
    State(app): State<AppState>,
    Query(page): Query<Page>,
) -> Result<Json<Vec<Customer>>, AppError> {
    let search = page
        .search
        .map(|s| format!("%{}%", s.trim().to_lowercase()));
    let rows = query(
        "SELECT p.id,p.username,p.active,p.created_at, \
           COUNT(d.id) AS devices, COUNT(d.id) FILTER (WHERE d.active AND d.paid_until>now()) AS paid_devices \
         FROM pharmacies p LEFT JOIN pharmacy_devices d ON d.pharmacy_id=p.id \
         WHERE ($1::uuid IS NULL OR p.id>$1) AND ($2::text IS NULL OR lower(p.username) LIKE $2) \
         GROUP BY p.id ORDER BY p.id LIMIT 100",
    )
    .bind(page.after)
    .bind(search)
    .fetch_all(&app.pool)
    .await?;
    let customers = rows
        .iter()
        .map(|row| {
            Ok(Customer {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                active: row.try_get("active")?,
                created_at: row.try_get("created_at")?,
                devices: row.try_get("devices")?,
                paid_devices: row.try_get("paid_devices")?,
            })
        })
        .collect::<Result<_, sqlx_core::error::Error>>()?;
    Ok(Json(customers))
}

pub async fn customer_devices(
    State(app): State<AppState>,
    Path(pharmacy_id): Path<Uuid>,
) -> Result<Json<Vec<Device>>, AppError> {
    let rows = query(
        "SELECT id,name,active,paid_until,last_seen_at FROM pharmacy_devices WHERE pharmacy_id=$1 ORDER BY id",
    )
    .bind(pharmacy_id)
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

#[derive(Debug, Deserialize)]
pub struct ExtendInput {
    pub days: i64,
}

pub async fn extend_device(
    State(app): State<AppState>,
    Path(device_id): Path<i64>,
    Json(input): Json<ExtendInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let paid_until = app
        .extend_device(device_id, input.days, "admin")
        .await
        .map_err(|_| AppError::BadRequest)?;
    let paid_until = paid_until
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| AppError::Internal)?;
    Ok(Json(serde_json::json!({ "paid_until": paid_until })))
}

#[derive(Debug, Deserialize)]
pub struct ActiveInput {
    pub active: bool,
}

pub async fn set_device_active(
    State(app): State<AppState>,
    Path(device_id): Path<i64>,
    Json(input): Json<ActiveInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = app.pool.begin().await?;
    let pharmacy_id: Option<Uuid> =
        query("UPDATE pharmacy_devices SET active=$2 WHERE id=$1 RETURNING pharmacy_id")
            .bind(device_id)
            .bind(input.active)
            .fetch_optional(&mut *tx)
            .await?
            .map(|row| row.try_get("pharmacy_id"))
            .transpose()?;
    let pharmacy_id = pharmacy_id.ok_or(AppError::NotFound)?;
    let action = if input.active {
        "device.activated"
    } else {
        "device.deactivated"
    };
    audit(
        &mut tx,
        Some(pharmacy_id),
        action,
        serde_json::json!({ "device_id": device_id }),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(serde_json::json!({ "active": input.active })))
}

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub pharmacy_id: Option<Uuid>,
    pub actor: String,
    pub action: String,
    pub detail: serde_json::Value,
    pub source: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct AuditPage {
    /// Keyset cursor, newest first: entries with an id below this.
    pub before: Option<i64>,
}

pub async fn audit_log(
    State(app): State<AppState>,
    Query(page): Query<AuditPage>,
) -> Result<Json<Vec<AuditEntry>>, AppError> {
    let rows = query(
        "SELECT id,pharmacy_id,actor,action,detail,source,created_at FROM audit_log \
         WHERE ($1::bigint IS NULL OR id<$1) ORDER BY id DESC LIMIT 100",
    )
    .bind(page.before)
    .fetch_all(&app.pool)
    .await?;
    let entries = rows
        .iter()
        .map(|row| {
            Ok(AuditEntry {
                id: row.try_get("id")?,
                pharmacy_id: row.try_get("pharmacy_id")?,
                actor: row.try_get("actor")?,
                action: row.try_get("action")?,
                detail: row.try_get("detail")?,
                source: row.try_get("source")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<Result<_, sqlx_core::error::Error>>()?;
    Ok(Json(entries))
}

async fn audit(
    tx: &mut sqlx_core::transaction::Transaction<'_, sqlx_postgres::Postgres>,
    pharmacy_id: Option<Uuid>,
    action: &str,
    detail: serde_json::Value,
) -> Result<(), AppError> {
    query("INSERT INTO audit_log(pharmacy_id,actor,action,detail,source) VALUES($1,'owner',$2,$3,'admin')")
        .bind(pharmacy_id)
        .bind(action)
        .bind(detail)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
