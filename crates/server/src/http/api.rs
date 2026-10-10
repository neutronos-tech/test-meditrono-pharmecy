use crate::{app::AppState, error::AppError};
use axum::{
    body::{Body, Bytes},
    extract::{Extension, Json, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, header},
    response::{IntoResponse, Response},
};
use contracts::{
    auth::PharmacyAccount,
    sync::{SyncRequest, SyncResponse},
};
use futures_util::stream;
use serde::Deserialize;

pub use crate::auth::api::{login, logout, me};

pub async fn health() -> &'static str {
    "ok"
}

fn ndjson_headers(revision: u64, count: Option<u64>) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ndjson"),
    );
    headers.insert(
        HeaderName::from_static("x-medicine-revision"),
        HeaderValue::from_str(&revision.to_string()).map_err(|_| AppError::Internal)?,
    );
    if let Some(count) = count {
        headers.insert(
            HeaderName::from_static("x-medicine-count"),
            HeaderValue::from_str(&count.to_string()).map_err(|_| AppError::Internal)?,
        );
    }
    Ok(headers)
}

pub async fn full_medicines(State(app): State<AppState>) -> Result<Response, AppError> {
    let (revision, count, medicines) = app.medicine_catalog().await.map_err(AppError::Database)?;
    let headers = ndjson_headers(revision, Some(count))?;
    let lines = medicines.into_iter().map(|medicine| {
        let mut bytes = serde_json::to_vec(&medicine).map_err(std::io::Error::other)?;
        bytes.push(b'\n');
        Ok::<Bytes, std::io::Error>(Bytes::from(bytes))
    });
    Ok((headers, Body::from_stream(stream::iter(lines))).into_response())
}

#[derive(Deserialize)]
pub struct ChangesQuery {
    after: u64,
}

pub async fn medicine_changes(
    State(app): State<AppState>,
    Query(query): Query<ChangesQuery>,
) -> Result<Response, AppError> {
    let (revision, changes) = app
        .medicine_changes(query.after)
        .await
        .map_err(AppError::Database)?;
    let headers = ndjson_headers(revision, None)?;
    let lines = changes.into_iter().map(|change| {
        let mut bytes = serde_json::to_vec(&change).map_err(std::io::Error::other)?;
        bytes.push(b'\n');
        Ok::<Bytes, std::io::Error>(Bytes::from(bytes))
    });
    Ok((headers, Body::from_stream(stream::iter(lines))).into_response())
}

pub async fn synchronize(
    State(app): State<AppState>,
    Extension(pharmacy): Extension<PharmacyAccount>,
    Json(request): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, AppError> {
    app.synchronize(&pharmacy, request).await.map(Json)
}
