use crate::{app::AppState, error::AppError};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

#[derive(Clone)]
pub struct BrowserCredential {
    pub token: String,
}

#[derive(Clone)]
pub struct ApiCredential {
    pub token: String,
}

pub async fn require_browser_pharmacy(
    State(app): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let Some(token) = cookie_value(request.headers(), "pharmacy_session") else {
        return Err(Redirect::to("/login").into_response());
    };
    let unsafe_method = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    );
    let result = if unsafe_method {
        let Some(csrf) = request
            .headers()
            .get("x-csrf-token")
            .and_then(|value| value.to_str().ok())
        else {
            return Err(AppError::Forbidden.into_response());
        };
        app.authenticate_browser_mutation(&token, csrf).await
    } else {
        app.authenticate_browser_session(&token).await
    };
    let pharmacy = result.map_err(|error| match error {
        crate::app::AuthenticateError::Csrf => AppError::Forbidden.into_response(),
        crate::app::AuthenticateError::Database => AppError::Internal.into_response(),
        crate::app::AuthenticateError::Invalid => Redirect::to("/login").into_response(),
    })?;
    request.extensions_mut().insert(pharmacy);
    request.extensions_mut().insert(BrowserCredential { token });
    Ok(next.run(request).await)
}

pub async fn require_api_pharmacy(
    State(app): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let Some(token) = bearer_token(request.headers()) else {
        return Err(AppError::Unauthorized.into_response());
    };
    let pharmacy = app
        .authenticate_session(&token)
        .await
        .map_err(|error| match error {
            crate::app::AuthenticateError::Invalid => AppError::Unauthorized.into_response(),
            _ => AppError::Internal.into_response(),
        })?;
    request.extensions_mut().insert(pharmacy);
    request.extensions_mut().insert(ApiCredential { token });
    Ok(next.run(request).await)
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("Bearer") && !token.is_empty()).then(|| token.to_string())
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    cookies.split(';').find_map(|cookie| {
        let (key, value) = cookie.trim().split_once('=')?;
        (key == name && !value.is_empty()).then(|| value.to_string())
    })
}
